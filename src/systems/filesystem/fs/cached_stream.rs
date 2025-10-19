use crate::extensions::api::{AttributeRow, FileRow};
use std::sync::Arc;

use super::super::{IndexKind, IndexStream, IndexUpdate, ProgressSnapshot};
use super::MAX_BATCH_SIZE;
use super::cache::CachedEntry;

pub(super) fn stream_cached_entry(
    entry: CachedEntry,
    preview_len: Option<usize>,
    tx: &std::sync::mpsc::Sender<super::super::IndexResult>,
) {
    let stream = IndexStream::new(tx, 0, IndexKind::Preview);
    let data = entry.data;
    let total_files = data.files.len();
    let total_attributes = data.attributes.len();

    if total_files == 0 && total_attributes == 0 {
        return;
    }

    let start_index = preview_len.unwrap_or(0).min(total_files);
    let attributes: Arc<[AttributeRow]> = data.attributes.into();
    let mut files = data.files;

    if start_index > 0 {
        files.drain(..start_index);
    }

    if files.is_empty() {
        let progress = ProgressSnapshot {
            indexed_attributes: total_attributes,
            indexed_files: total_files,
            total_attributes: Some(total_attributes),
            total_files: Some(total_files),
            complete: false,
        };

        let _ = stream.send_update(
            IndexUpdate {
                files: Arc::from(Vec::<FileRow>::new()),
                attributes,
                progress,
                reset: preview_len.is_none(),
                cached_data: None,
            },
            false,
        );
        return;
    }

    let mut dispatched = start_index;
    let mut first_batch = true;

    while !files.is_empty() {
        let chunk_len = files.len().min(MAX_BATCH_SIZE);
        let chunk: Vec<FileRow> = files.drain(..chunk_len).collect();
        dispatched += chunk_len;

        let progress = ProgressSnapshot {
            indexed_attributes: total_attributes,
            indexed_files: dispatched,
            total_attributes: Some(total_attributes),
            total_files: Some(total_files),
            complete: false,
        };

        let update = IndexUpdate {
            files: chunk.into(),
            attributes: attributes.clone(),
            progress,
            reset: preview_len.is_none() && first_batch,
            cached_data: None,
        };

        if !stream.send_update(update, false) {
            break;
        }

        first_batch = false;
    }
}
