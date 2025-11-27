//! Search pipeline and results aggregation.
//!
//! This feature module contains the fuzzy matching engine, scoring aggregation,
//! and streaming infrastructure that powers the search experience.

mod channel;
mod data;
mod file;
mod fs;
mod matcher;
pub(crate) mod runtime;

pub use channel::{MatchBatch, SearchMarker, SearchResult, SearchStream, SearchView, SearchViewV2};
pub use data::{FILES_DATASET_KEY, SearchData};
pub use file::{FileRow, SearchOutcome, SearchSelection, TruncationStyle};
pub use fs::{Fs, FsIter, OsFs};
pub use matcher::{config_for_query, stream_files};

/// Tunable thresholds shared across the search pipeline.
pub const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;

/// Maximum number of rows rendered in the result table.
pub const MAX_RENDERED_RESULTS: usize = 2_000;

/// Number of matches processed per scoring chunk.
pub const MATCH_CHUNK_SIZE: usize = 512;

/// Number of rows processed before emitting a heartbeat for empty queries.
pub const EMPTY_QUERY_BATCH: usize = 128;

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
