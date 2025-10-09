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

    let mut data = SearchData::new();
    if options.context_label.is_none() {
        options.context_label = Some(root.display().to_string());
    }
    data.context_label = options.context_label.clone();

    thread::spawn(move || {
        let (file_tx, file_rx) = mpsc::channel::<FileRow>();
        let walker_root = Arc::new(root);
        let threads = options
            .threads
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
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

        let aggregator = thread::spawn(move || {
            let mut batcher = UpdateBatcher::new();

            while let Ok(file) = file_rx.recv() {
                batcher.record_file(file);

                if batcher.should_flush() && batcher.flush(&update_tx, false).is_err() {
                    return;
                }
            }

            let _ = batcher.flush(&update_tx, true);
        });

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
                Box::new(move |entry: Result<DirEntry, IgnoreError>| {
                    if let Ok(entry) = entry {
                        let Some(file_type) = entry.file_type() else {
                            return WalkState::Continue;
                        };
                        if !file_type.is_file() {
                            return WalkState::Continue;
                        }

                        let path = entry.path();
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
                        let relative_display = relative.to_string_lossy().replace("\\", "/");
                        let file = FileRow::new(relative_display, tags);
                        if sender.send(file).is_err() {
                            return WalkState::Quit;
                        }
                    }

                    WalkState::Continue
                })
            });

        drop(file_tx);
        let _ = aggregator.join();
    });

    Ok((data, rx))
}

struct UpdateBatcher {
    facet_counts: BTreeMap<String, usize>,
    pending_facets: BTreeMap<String, usize>,
    pending_files: Vec<FileRow>,
    indexed_files: usize,
    last_dispatch: Instant,
}

impl UpdateBatcher {
    fn new() -> Self {
        Self {
            facet_counts: BTreeMap::new(),
            pending_facets: BTreeMap::new(),
            pending_files: Vec::new(),
            indexed_files: 0,
            last_dispatch: Instant::now(),
        }
    }

    fn record_file(&mut self, file: FileRow) {
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

        if self.pending_files.is_empty() && self.pending_facets.is_empty() {
            return false;
        }

        self.last_dispatch.elapsed() >= DISPATCH_INTERVAL
    }

    fn flush(
        &mut self,
        tx: &Sender<IndexUpdate>,
        complete: bool,
    ) -> Result<(), mpsc::SendError<IndexUpdate>> {
        if !complete && self.pending_files.is_empty() && self.pending_facets.is_empty() {
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

        tx.send(IndexUpdate {
            files,
            facets,
            progress,
        })?;

        self.last_dispatch = Instant::now();
        Ok(())
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
