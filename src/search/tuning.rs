/// Tunable thresholds shared across the search pipeline.
pub const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;

/// Maximum number of rows rendered in the result table.
pub const MAX_RENDERED_RESULTS: usize = 2_000;

/// Number of matches processed per scoring chunk.
pub const MATCH_CHUNK_SIZE: usize = 512;

/// Number of rows processed before emitting a heartbeat for empty queries.
pub const EMPTY_QUERY_BATCH: usize = 128;
