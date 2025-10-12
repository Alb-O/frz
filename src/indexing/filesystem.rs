use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use ignore::{DirEntry, Error as IgnoreError, WalkBuilder, WalkState};

use crate::types::tags_for_relative_path;
use crate::types::{FacetRow, FileRow, SearchData};

use super::{IndexUpdate, ProgressSnapshot};

#[path = "filesystem_cache.rs"]
mod cache;
use cache::CacheHandle;
use cache::CacheWriter;

const MIN_BATCH_SIZE: usize = 32;
const MAX_BATCH_SIZE: usize = 1_024;
const DISPATCH_INTERVAL: Duration = Duration::from_millis(120);

#[derive(Debug, Clone)]
pub struct FilesystemOptions {
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub respect_ignore_files: bool,
    pub git_ignore: bool,
    pub git_global: bool,
    pub git_exclude: bool,
    pub global_ignores: Vec<String>,
    pub threads: Option<usize>,
    pub max_depth: Option<usize>,
    pub allowed_extensions: Option<Vec<String>>,
    pub context_label: Option<String>,
}

impl Default for FilesystemOptions {
    fn default() -> Self {
        Self {
            include_hidden: true,
            follow_symlinks: false,
            respect_ignore_files: true,
            git_ignore: true,
            git_global: true,
            git_exclude: true,
            global_ignores: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
                ".cache".to_string(),
            ],
            threads: None,
            max_depth: None,
            allowed_extensions: None,
            context_label: None,
        }
    }
}

pub(crate) fn spawn_filesystem_index(
    root: PathBuf,
    mut options: FilesystemOptions,
) -> Result<(SearchData, Receiver<IndexUpdate>)> {
    let (tx, rx) = mpsc::channel();

    let cache_handle = CacheHandle::resolve(&root, &options);
    let mut data = SearchData::new();

    if options.context_label.is_none() {
        options.context_label = Some(root.display().to_string());
    }
    data.context_label = options.context_label.clone();

    let should_reset = cache_handle.is_some();
    let cache_handle_for_thread = cache_handle.clone();
    let context_label = options.context_label.clone();

    thread::spawn(move || {
        let mut reindex_delay = Duration::ZERO;

        if let Some(handle) = cache_handle_for_thread.as_ref() {
            if let Some(entry) = handle.load() {
                reindex_delay = entry.reindex_delay();

                let mut data = entry.data;
                if data.context_label.is_none() {
                    data.context_label = context_label.clone();
                }

                let files: Arc<[FileRow]> = data.files.clone().into();
                let facets: Arc<[FacetRow]> = data.facets.clone().into();
                let progress = ProgressSnapshot {
                    indexed_facets: facets.len(),
                    indexed_files: files.len(),
                    total_facets: Some(facets.len()),
                    total_files: Some(files.len()),
                    complete: false,
                };

                if !files.is_empty() || !facets.is_empty() {
                    let _ = tx.send(IndexUpdate {
                        files,
                        facets,
                        progress,
                        reset: true,
                        cached_data: Some(data),
                    });
                }
            }
        }

        if !reindex_delay.is_zero() {
            thread::sleep(reindex_delay);
        }

        let (file_tx, file_rx) = mpsc::channel::<FileRow>();
        let walker_root = Arc::new(root);
        let threads = options
            .threads
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| {
                std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
            });
        let extension_filter = options.allowed_extensions.as_ref().map(|extensions| {
            Arc::new(
                extensions
                    .iter()
                    .map(|ext| normalize_extension(ext))
                    .filter(|ext| !ext.is_empty())
                    .collect::<HashSet<_>>(),
            )
        });
        let update_tx = tx;

        let cache_writer = cache_handle_for_thread
            .as_ref()
            .and_then(|handle| handle.writer(context_label.clone()));
        let aggregator = thread::spawn(move || {
            let mut batcher = UpdateBatcher::new(should_reset, cache_writer);

            while let Ok(file) = file_rx.recv() {
                batcher.record_file(file);

                if batcher.should_flush() && batcher.flush(&update_tx, false).is_err() {
                    return None;
                }
            }

            batcher.finalize(&update_tx).unwrap_or_default()
        });

        let global_ignores = options.global_ignores.clone();

        WalkBuilder::new(walker_root.as_path())
            .hidden(!options.include_hidden)
            .follow_links(options.follow_symlinks)
            .git_ignore(options.git_ignore)
            .git_global(options.git_global)
            .git_exclude(options.git_exclude)
            .ignore(options.respect_ignore_files)
            .parents(true)
            .threads(threads)
            .max_depth(options.max_depth)
            .build_parallel()
            .run(|| {
                let sender = file_tx.clone();
                let root = Arc::clone(&walker_root);
                let extension_filter = extension_filter.clone();
                let global_ignores = global_ignores.clone();
                Box::new(move |entry: Result<DirEntry, IgnoreError>| {
                    if let Ok(entry) = entry {
                        let Some(file_type) = entry.file_type() else {
                            return WalkState::Continue;
                        };
                        if !file_type.is_file() {
                            return WalkState::Continue;
                        }

                        let path = entry.path();
                        // Skip global ignore paths
                        if path.components().any(|comp| {
                            comp.as_os_str()
                                .to_str()
                                .map(|s| global_ignores.iter().any(|g| g == s))
                                .unwrap_or(false)
                        }) {
                            return WalkState::Continue;
                        }
                        let relative = path.strip_prefix(root.as_path()).unwrap_or(path);
                        if let Some(filter) = extension_filter.as_ref() {
                            let extension = relative
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext.to_ascii_lowercase());
                            if extension.as_ref().is_none_or(|ext| !filter.contains(ext)) {
                                return WalkState::Continue;
                            }
                        }
                        let tags = tags_for_relative_path(relative);
                        let relative_display = relative.to_string_lossy().replace('\\', "/");
                        let file = FileRow::new(relative_display, tags);
                        if sender.send(file).is_err() {
                            return WalkState::Quit;
                        }
                    }

                    WalkState::Continue
                })
            });

        drop(file_tx);
        if let Ok(Some(writer)) = aggregator.join() {
            let _ = writer.finish();
        }
    });

    Ok((data, rx))
}

struct UpdateBatcher {
    facet_counts: BTreeMap<String, usize>,
    pending_facets: BTreeMap<String, usize>,
    pending_files: Vec<FileRow>,
    indexed_files: usize,
    last_dispatch: Instant,
    emit_reset: bool,
    cache_writer: Option<CacheWriter>,
}

impl UpdateBatcher {
    fn new(emit_reset: bool, cache_writer: Option<CacheWriter>) -> Self {
        Self {
            facet_counts: BTreeMap::new(),
            pending_facets: BTreeMap::new(),
            pending_files: Vec::new(),
            indexed_files: 0,
            last_dispatch: Instant::now(),
            emit_reset,
            cache_writer,
        }
    }

    fn record_file(&mut self, file: FileRow) {
        if let Some(writer) = &mut self.cache_writer {
            writer.record(&file);
        }

        for tag in &file.tags {
            let count = self.facet_counts.entry(tag.clone()).or_insert(0);
            *count += 1;
            self.pending_facets.insert(tag.clone(), *count);
        }

        self.indexed_files += 1;
        self.pending_files.push(file);
    }

    fn should_flush(&self) -> bool {
        if self.pending_files.len() >= batch_size_for(self.indexed_files) {
            return true;
        }

        if !self.emit_reset && self.pending_files.is_empty() && self.pending_facets.is_empty() {
            return false;
        }

        self.last_dispatch.elapsed() >= DISPATCH_INTERVAL
    }

    fn flush(
        &mut self,
        tx: &Sender<IndexUpdate>,
        complete: bool,
    ) -> Result<(), mpsc::SendError<IndexUpdate>> {
        if !complete
            && !self.emit_reset
            && self.pending_files.is_empty()
            && self.pending_facets.is_empty()
        {
            return Ok(());
        }

        let files_vec = std::mem::take(&mut self.pending_files);
        let files: Arc<[FileRow]> = files_vec.into();
        let facets: Arc<[FacetRow]> = if self.pending_facets.is_empty() {
            Arc::default()
        } else {
            let collected: Vec<FacetRow> = self
                .pending_facets
                .iter()
                .map(|(name, count)| FacetRow::new(name.clone(), *count))
                .collect();
            self.pending_facets.clear();
            collected.into()
        };

        let progress = ProgressSnapshot {
            indexed_facets: self.facet_counts.len(),
            indexed_files: self.indexed_files,
            total_facets: complete.then_some(self.facet_counts.len()),
            total_files: complete.then_some(self.indexed_files),
            complete,
        };

        let reset = self.emit_reset;
        if self.emit_reset {
            self.emit_reset = false;
        }

        tx.send(IndexUpdate {
            files,
            facets,
            progress,
            reset,
            cached_data: None,
        })?;

        self.last_dispatch = Instant::now();
        Ok(())
    }

    fn finalize(
        self,
        tx: &Sender<IndexUpdate>,
    ) -> Result<Option<CacheWriter>, mpsc::SendError<IndexUpdate>> {
        let mut this = self;
        this.flush(tx, true)?;
        Ok(this.cache_writer)
    }
}

fn batch_size_for(indexed_files: usize) -> usize {
    if indexed_files < 1_024 {
        MIN_BATCH_SIZE
    } else if indexed_files < 16_384 {
        256
    } else {
        MAX_BATCH_SIZE
    }
}

fn normalize_extension(ext: &str) -> String {
    ext.trim().trim_start_matches('.').to_ascii_lowercase()
}
