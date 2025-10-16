//! Public-facing search system APIs that plugins can opt into using.

pub use super::commands::SearchStream;
pub use super::streaming::{stream_facets, stream_files};

/// Threshold after which pre-filtering should be enabled for large data sets.
pub const PREFILTER_ENABLE_THRESHOLD: usize = super::PREFILTER_ENABLE_THRESHOLD;

/// Maximum number of results that the UI will attempt to render at once.
pub const MAX_RENDERED_RESULTS: usize = super::MAX_RENDERED_RESULTS;
