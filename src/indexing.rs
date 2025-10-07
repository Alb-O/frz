#[cfg(feature = "fs")]
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(feature = "fs"))]
use std::sync::mpsc::Receiver;
#[cfg(feature = "fs")]
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(feature = "fs")]
use std::thread;

use anyhow::Result;
#[cfg(not(feature = "fs"))]
use anyhow::bail;

#[cfg(feature = "fs")]
use ignore::{DirEntry, Error as IgnoreError, WalkBuilder, WalkState};
#[cfg(feature = "fs")]
use std::time::{Duration, Instant};

#[cfg(feature = "fs")]
use crate::types::tags_for_relative_path;
use crate::types::{FacetRow, FileRow, SearchData};

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[cfg_attr(not(feature = "fs"), allow(dead_code))]
#[derive(Debug, Clone)]
pub(crate) struct IndexUpdate {
    pub(crate) files: Arc<[FileRow]>,
    pub(crate) facets: Arc<[FacetRow]>,
    pub(crate) progress: ProgressSnapshot,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[cfg_attr(not(feature = "fs"), allow(dead_code))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProgressSnapshot {
    pub(crate) indexed_facets: usize,
    pub(crate) indexed_files: usize,
    pub(crate) total_facets: Option<usize>,
    pub(crate) total_files: Option<usize>,
    pub(crate) complete: bool,
}

#[cfg(feature = "fs")]
const MIN_BATCH_SIZE: usize = 32;
#[cfg(feature = "fs")]
const MAX_BATCH_SIZE: usize = 1024;
#[cfg(feature = "fs")]
const DISPATCH_INTERVAL: Duration = Duration::from_millis(120);

#[cfg(feature = "fs")]
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
            let mut facet_counts: BTreeMap<String, usize> = BTreeMap::new();
            let mut pending_facets: BTreeMap<String, usize> = BTreeMap::new();
            let mut pending_files: Vec<FileRow> = Vec::new();
            let mut indexed_files: usize = 0;
            let mut last_dispatch = Instant::now();

            while let Ok(file) = file_rx.recv() {
                for tag in &file.tags {
                    let count = facet_counts.entry(tag.clone()).or_insert(0);
                    *count += 1;
                    pending_facets.insert(tag.clone(), *count);
                }

                pending_files.push(file);
                indexed_files += 1;
                let batch_size = batch_size_for(indexed_files);
                let should_flush = pending_files.len() >= batch_size
                    || (!pending_files.is_empty() || !pending_facets.is_empty())
                        && last_dispatch.elapsed() >= DISPATCH_INTERVAL;

                if should_flush
                    && dispatch_update(
                        &update_tx,
                        &mut pending_files,
                        &mut pending_facets,
                        &facet_counts,
                        indexed_files,
                        false,
                    )
                    .is_err()
                {
                    return;
                }

                if should_flush {
                    last_dispatch = Instant::now();
                }
            }

            let _ = dispatch_update(
                &update_tx,
                &mut pending_files,
                &mut pending_facets,
                &facet_counts,
                indexed_files,
                true,
            );
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

#[cfg(not(feature = "fs"))]
#[allow(dead_code)]
pub(crate) fn spawn_filesystem_index(
    _root: PathBuf,
) -> Result<(SearchData, Receiver<IndexUpdate>)> {
    bail!("filesystem support is disabled; enable the `fs` feature");
}

#[cfg(feature = "fs")]
fn dispatch_update(
    tx: &Sender<IndexUpdate>,
    pending_files: &mut Vec<FileRow>,
    pending_facets: &mut BTreeMap<String, usize>,
    facet_counts: &BTreeMap<String, usize>,
    indexed_files: usize,
    complete: bool,
) -> Result<(), mpsc::SendError<IndexUpdate>> {
    if !complete && pending_files.is_empty() && pending_facets.is_empty() {
        return Ok(());
    }

    let files_vec = std::mem::take(pending_files);
    let files: Arc<[FileRow]> = files_vec.into();
    let facets: Arc<[FacetRow]> = if pending_facets.is_empty() {
        Arc::default()
    } else {
        let collected: Vec<FacetRow> = pending_facets
            .iter()
            .map(|(name, count)| FacetRow::new(name.clone(), *count))
            .collect();
        pending_facets.clear();
        collected.into()
    };

    let progress = ProgressSnapshot {
        indexed_facets: facet_counts.len(),
        indexed_files,
        total_facets: complete.then_some(facet_counts.len()),
        total_files: complete.then_some(indexed_files),
        complete,
    };

    tx.send(IndexUpdate {
        files,
        facets,
        progress,
    })
}

#[cfg(feature = "fs")]
fn batch_size_for(indexed_files: usize) -> usize {
    if indexed_files < 1_024 {
        MIN_BATCH_SIZE
    } else if indexed_files < 16_384 {
        256
    } else {
        MAX_BATCH_SIZE
    }
}

#[cfg(feature = "fs")]
pub(crate) fn merge_update(data: &mut SearchData, update: &IndexUpdate) {
    if !update.files.is_empty() {
        data.files.extend(update.files.iter().cloned());
    }

    if !update.facets.is_empty() {
        for facet in update.facets.iter() {
            match data
                .facets
                .binary_search_by(|existing| existing.name.cmp(&facet.name))
            {
                Ok(index) => data.facets[index].count = facet.count,
                Err(index) => data.facets.insert(index, facet.clone()),
            }
        }
    }
}
