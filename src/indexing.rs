use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::Result;
use walkdir::WalkDir;

use crate::types::{FacetRow, FileRow, SearchData};

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[derive(Debug, Clone)]
pub(crate) struct IndexUpdate {
    pub(crate) files: Vec<FileRow>,
    pub(crate) facets: Vec<FacetRow>,
    pub(crate) progress: ProgressSnapshot,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProgressSnapshot {
    pub(crate) indexed_facets: usize,
    pub(crate) indexed_files: usize,
    pub(crate) total_facets: Option<usize>,
    pub(crate) total_files: Option<usize>,
    pub(crate) complete: bool,
}

const BATCH_SIZE: usize = 50;

pub(crate) fn spawn_filesystem_index(root: PathBuf) -> Result<(SearchData, Receiver<IndexUpdate>)> {
    let (tx, rx) = mpsc::channel();

    let mut data = SearchData::new();
    data.context_label = Some(root.display().to_string());

    thread::spawn(move || {
        let mut facet_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut pending_facets: BTreeMap<String, usize> = BTreeMap::new();
        let mut pending_files: Vec<FileRow> = Vec::new();
        let mut indexed_files: usize = 0;

        let walker = WalkDir::new(&root).into_iter();
        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let relative = path.strip_prefix(&root).unwrap_or(path);
            let mut tags: BTreeSet<String> = BTreeSet::new();

            if let Some(parent) = relative.parent() {
                for component in parent.components() {
                    if let Component::Normal(part) = component {
                        let value = part.to_string_lossy().to_string();
                        if !value.is_empty() {
                            tags.insert(value);
                        }
                    }
                }
            }

            if let Some(ext) = relative.extension().and_then(|ext| ext.to_str())
                && !ext.is_empty()
            {
                tags.insert(format!("*.{ext}"));
            }

            let tags_vec: Vec<String> = tags.into_iter().collect();
            for tag in &tags_vec {
                let count = facet_counts.entry(tag.clone()).or_insert(0);
                *count += 1;
                pending_facets.insert(tag.clone(), *count);
            }

            let relative_display = relative.to_string_lossy().replace("\\", "/");
            pending_files.push(FileRow::new(relative_display, tags_vec));
            indexed_files += 1;

            if pending_files.len() >= BATCH_SIZE {
                if dispatch_update(
                    &tx,
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
            }
        }

        let _ = dispatch_update(
            &tx,
            &mut pending_files,
            &mut pending_facets,
            &facet_counts,
            indexed_files,
            true,
        );
    });

    Ok((data, rx))
}

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

    let files = pending_files.drain(..).collect();
    let facets = pending_facets
        .iter()
        .map(|(name, count)| FacetRow::new(name.clone(), *count))
        .collect();
    pending_facets.clear();

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

pub(crate) fn merge_update(data: &mut SearchData, update: &IndexUpdate) {
    if !update.files.is_empty() {
        data.files.extend(update.files.iter().cloned());
    }

    if !update.facets.is_empty() {
        for facet in &update.facets {
            if let Some(existing) = data
                .facets
                .iter_mut()
                .find(|existing| existing.name == facet.name)
            {
                existing.count = facet.count;
            } else {
                data.facets.push(facet.clone());
            }
        }
    }
}
