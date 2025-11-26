mod fs;
pub mod stream;

use std::sync::Arc;

pub use fs::{FilesystemOptions, spawn_filesystem_index};
pub use stream::{IndexKind, IndexResult, IndexStream, IndexView};

use crate::search::{FileRow, SearchData};

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[derive(Debug, Clone)]
pub struct IndexUpdate {
	pub files: Arc<[FileRow]>,
	pub progress: ProgressSnapshot,
	pub reset: bool,
	pub cached_data: Option<SearchData>,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[derive(Debug, Clone, Copy)]
pub struct ProgressSnapshot {
	pub indexed_files: usize,
	pub total_files: Option<usize>,
	pub complete: bool,
}

/// Merge an index update into the search data, resetting if indicated.
pub fn merge_update(data: &mut SearchData, update: &IndexUpdate) {
	if update.reset {
		data.files.clear();
	}

	if !update.files.is_empty() {
		data.files.extend(update.files.iter().cloned());
	}
}
