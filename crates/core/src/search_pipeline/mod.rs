//! Search pipeline and results aggregation.
//!
//! This feature module contains the fuzzy matching engine, scoring aggregation,
//! and streaming infrastructure that powers the search experience.

use std::sync::Arc;
use std::sync::atomic::AtomicU64;

mod data;
mod file;
mod fs;
pub mod runtime;

pub use data::{FILES_DATASET_KEY, SearchData};
pub use file::{FileRow, SearchOutcome, SearchSelection, TruncationStyle};
pub use frz_stream::search::{
	Dataset, EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, MAX_RENDERED_RESULTS, MatchBatch,
	PREFILTER_ENABLE_THRESHOLD, SearchMarker, SearchResult, SearchStream, SearchView, SearchViewV2,
	config_for_query,
};
pub use fs::{Fs, FsIter, OsFs};

/// Streams file matches for the given query back to the UI thread.
pub fn stream_files(
	data: &SearchData,
	query: &str,
	stream: SearchStream<'_>,
	latest_query_id: &Arc<AtomicU64>,
) -> bool {
	struct FileDataset<'a>(&'a [FileRow]);

	impl<'a> Dataset for FileDataset<'a> {
		fn len(&self) -> usize {
			self.0.len()
		}

		fn key_for(&self, index: usize) -> &str {
			self.0[index].search_text()
		}
	}

	let files = FileDataset(data.files.as_slice());
	frz_stream::search::stream_dataset(&files, query, stream, latest_query_id, move |index| {
		files.0[index].path.clone()
	})
}

/// Compute a stable 64-bit hash for the provided value.
///
/// This uses a simple FNV-1a implementation to avoid pulling in
/// additional dependencies while guaranteeing deterministic output across
/// processes and platforms.
#[must_use]
pub fn stable_hash64(value: &str) -> u64 {
	const FNV_OFFSET: u64 = 0xcbf29ce484222325;
	const FNV_PRIME: u64 = 0x00000100000001b3;

	let mut hash = FNV_OFFSET;
	for byte in value.as_bytes() {
		hash ^= u64::from(*byte);
		hash = hash.wrapping_mul(FNV_PRIME);
	}
	hash
}
