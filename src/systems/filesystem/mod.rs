mod fs;

use std::sync::Arc;

use crate::extensions::api::{AttributeRow, FileRow, SearchData};

pub use fs::FilesystemOptions;
pub use fs::spawn_filesystem_index;
pub mod extension;

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[derive(Debug, Clone)]
pub struct IndexUpdate {
    pub files: Arc<[FileRow]>,
    pub attributes: Arc<[AttributeRow]>,
    pub progress: ProgressSnapshot,
    pub reset: bool,
    pub cached_data: Option<SearchData>,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[derive(Debug, Clone, Copy)]
pub struct ProgressSnapshot {
    pub indexed_attributes: usize,
    pub indexed_files: usize,
    pub total_attributes: Option<usize>,
    pub total_files: Option<usize>,
    pub complete: bool,
}

pub fn merge_update(data: &mut SearchData, update: &IndexUpdate) {
    if update.reset {
        data.files.clear();
        data.attributes.clear();
    }

    if !update.files.is_empty() {
        data.files.extend(update.files.iter().cloned());
    }

    if !update.attributes.is_empty() {
        for attribute in update.attributes.iter() {
            match data
                .attributes
                .binary_search_by(|existing| existing.name.cmp(&attribute.name))
            {
                Ok(index) => data.attributes[index].count = attribute.count,
                Err(index) => data.attributes.insert(index, attribute.clone()),
            }
        }
    }
}
