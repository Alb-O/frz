use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::types::{FacetRow, FileRow};

use super::super::{IndexUpdate, ProgressSnapshot};
use super::MAX_BATCH_SIZE;
use super::cache::CachedEntry;

pub(super) fn stream_cached_entry(
    entry: CachedEntry,
    preview_len: Option<usize>,
    tx: &Sender<IndexUpdate>,
) {
    let data = entry.data;
    let total_files = data.files.len();
    let total_facets = data.facets.len();

    if total_files == 0 && total_facets == 0 {
        return;
    }

    let start_index = preview_len.unwrap_or(0).min(total_files);
    let facets: Arc<[FacetRow]> = data.facets.into();
    let mut files = data.files;

    if start_index > 0 {
        files.drain(..start_index);
    }

    if files.is_empty() {
        let progress = ProgressSnapshot {
            indexed_facets: total_facets,
            indexed_files: total_files,
            total_facets: Some(total_facets),
            total_files: Some(total_files),
            complete: false,
        };

        let _ = tx.send(IndexUpdate {
            files: Arc::from(Vec::<FileRow>::new()),
            facets,
            progress,
            reset: preview_len.is_none(),
            cached_data: None,
        });
        return;
    }

    let mut dispatched = start_index;
    let mut first_batch = true;

    while !files.is_empty() {
        let chunk_len = files.len().min(MAX_BATCH_SIZE);
        let chunk: Vec<FileRow> = files.drain(..chunk_len).collect();
        dispatched += chunk_len;

        let progress = ProgressSnapshot {
            indexed_facets: total_facets,
            indexed_files: dispatched,
            total_facets: Some(total_facets),
            total_files: Some(total_files),
            complete: false,
        };

        let update = IndexUpdate {
            files: chunk.into(),
            facets: facets.clone(),
            progress,
            reset: preview_len.is_none() && first_batch,
            cached_data: None,
        };

        if tx.send(update).is_err() {
            break;
        }

        first_batch = false;
    }
}
