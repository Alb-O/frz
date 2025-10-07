use std::collections::BTreeMap;
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

pub(crate) fn spawn_filesystem_index(root: PathBuf) -> Result<(SearchData, Receiver<IndexUpdate>)> {
    let (tx, rx) = mpsc::channel();

    let mut data = SearchData::new();
    data.context_label = Some(root.display().to_string());

    thread::spawn(move || {
        let (file_tx, file_rx) = mpsc::channel::<FileRow>();
        let walker_root = Arc::new(root);
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        let update_tx = tx;

        let aggregator = thread::spawn(move || {
            let mut batcher = UpdateBatcher::new();

            while let Ok(file) = file_rx.recv() {
                batcher.record_file(file);

                if batcher.should_flush() {
                    if batcher.flush(&update_tx, false).is_err() {
                        return;
                    }
                }
            }

            let _ = batcher.flush(&update_tx, true);
        });

        WalkBuilder::new(walker_root.as_path())
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .ignore(true)
            .parents(true)
            .threads(threads)
            .build_parallel()
            .run(|| {
                let sender = file_tx.clone();
                let root = Arc::clone(&walker_root);
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
