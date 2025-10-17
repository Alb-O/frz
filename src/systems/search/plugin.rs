//! Public-facing search system APIs that plugins can opt into using.

pub use crate::plugins::api::search::config_for_query;
pub use crate::plugins::api::{SearchStream, stream_attributes, stream_files};

/// Threshold after which pre-filtering should be enabled for large data sets.
pub const PREFILTER_ENABLE_THRESHOLD: usize =
    crate::plugins::api::search::PREFILTER_ENABLE_THRESHOLD;

/// Maximum number of results that the UI will attempt to render at once.
pub const MAX_RENDERED_RESULTS: usize = crate::plugins::api::search::MAX_RENDERED_RESULTS;
