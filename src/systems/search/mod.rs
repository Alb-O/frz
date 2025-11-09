mod worker;

pub(crate) use crate::extensions::api::SearchResult;
use crate::extensions::api::{SearchData, SearchMode, StreamAction};

pub(crate) use worker::spawn;

// Re-exports for extension use
pub use crate::extensions::api::search::config_for_query;
pub use crate::extensions::api::{SearchStream, stream_attributes, stream_files};

/// Threshold after which pre-filtering should be enabled for large data sets.
pub const PREFILTER_ENABLE_THRESHOLD: usize =
    crate::extensions::api::search::PREFILTER_ENABLE_THRESHOLD;

/// Maximum number of results that the UI will attempt to render at once.
pub const MAX_RENDERED_RESULTS: usize = crate::extensions::api::search::MAX_RENDERED_RESULTS;

/// Commands understood by the background search worker.
#[derive(Debug)]
pub(crate) enum SearchCommand {
    /// Run a fuzzy search for the provided query and mode.
    Query {
        /// Identifier that allows the UI to correlate responses with the originating query.
        id: u64,
        /// User supplied query string.
        query: String,
        /// Which data set to search.
        mode: SearchMode,
    },
    /// Merge a fresh index update into the existing in-memory search data.
    Update(StreamAction<SearchData>),
    /// Stop the background worker thread.
    Shutdown,
}
