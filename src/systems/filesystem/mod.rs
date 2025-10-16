mod fs;

use std::sync::Arc;

use frz_plugin_api::{FacetRow, FileRow, SearchData};

pub use fs::FilesystemOptions;
pub use fs::spawn_filesystem_index;
pub mod plugin;

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[derive(Debug, Clone)]
pub struct IndexUpdate {
    pub files: Arc<[FileRow]>,
    pub facets: Arc<[FacetRow]>,
    pub progress: ProgressSnapshot,
    pub reset: bool,
    pub cached_data: Option<SearchData>,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[derive(Debug, Clone, Copy)]
pub struct ProgressSnapshot {
    pub indexed_facets: usize,
    pub indexed_files: usize,
    pub total_facets: Option<usize>,
    pub total_files: Option<usize>,
    pub complete: bool,
}

pub fn merge_update(data: &mut SearchData, update: &IndexUpdate) {
    if update.reset {
        data.files.clear();
        data.facets.clear();
    }

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
