//! Filesystem indexing and traversal.
//!
//! This feature module contains the filesystem walker, caching layer,
//! and streaming infrastructure for discovering and indexing files.

mod cache;
mod cached_stream;
mod options;
/// Streaming types for emitting index updates to the UI.
pub mod stream;
mod traversal;
mod update_batcher;

use std::sync::Arc;
use std::time::Duration;

pub use options::FilesystemOptions;
pub use stream::{IndexKind, IndexResult, IndexStream, IndexView};
pub(crate) use traversal::build_walk;
pub use traversal::spawn_filesystem_index;

use crate::filesystem::search::{FileRow, SearchData};

pub(crate) const MIN_BATCH_SIZE: usize = 32;
pub(crate) const MAX_BATCH_SIZE: usize = 1_024;
pub(crate) const DISPATCH_INTERVAL: Duration = Duration::from_millis(120);

/// Updates emitted by the filesystem indexer as it discovers new entries.
#[derive(Debug, Clone)]
pub struct IndexUpdate {
	/// Batch of newly discovered files.
	pub files: Arc<[FileRow]>,
	/// Current indexing progress for UI display.
	pub progress: ProgressSnapshot,
	/// Whether the consumer should clear existing data before applying this update.
	pub reset: bool,
	/// Complete search data snapshot from cache, if available.
	pub cached_data: Option<SearchData>,
}

/// Snapshot of the indexing progress suitable for updating the UI tracker.
#[derive(Debug, Clone, Copy)]
pub struct ProgressSnapshot {
	/// Number of files indexed so far.
	pub indexed_files: usize,
	/// Total number of files if known (e.g., from a completed cache).
	pub total_files: Option<usize>,
	/// Whether the indexing pass has finished.
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
