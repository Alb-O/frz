#[cfg(feature = "fs")]
mod filesystem;

use std::sync::Arc;

use crate::types::{FacetRow, FileRow, SearchData};

#[cfg(not(feature = "fs"))]
use std::path::PathBuf;
#[cfg(not(feature = "fs"))]
use std::sync::mpsc::Receiver;

#[cfg(not(feature = "fs"))]
use anyhow::Result;
#[cfg(not(feature = "fs"))]
use anyhow::bail;

#[cfg(feature = "fs")]
pub use filesystem::FilesystemOptions;
#[cfg(feature = "fs")]
pub use filesystem::spawn_filesystem_index;

#[cfg(feature = "fs")]
pub mod plugin;

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[cfg_attr(not(feature = "fs"), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct IndexUpdate {
    pub files: Arc<[FileRow]>,
    pub facets: Arc<[FacetRow]>,
    pub progress: ProgressSnapshot,
    pub reset: bool,
    pub cached_data: Option<SearchData>,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[cfg_attr(not(feature = "fs"), allow(dead_code))]
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

#[cfg(not(feature = "fs"))]
pub fn spawn_filesystem_index(_root: PathBuf) -> Result<(SearchData, Receiver<IndexUpdate>)> {
    bail!("filesystem support is disabled; enable the `fs` feature");
}
