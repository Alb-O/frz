use crate::types::SearchMode;

#[cfg(feature = "fs")]
use crate::indexing::IndexUpdate;

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
    #[cfg(feature = "fs")]
    Update(IndexUpdate),
    /// Stop the background worker thread.
    Shutdown,
}

/// Aggregated search results emitted back to the UI layer.
#[derive(Debug)]
pub(crate) struct SearchResult {
    /// Identifier matching the [`SearchCommand::Query`] that produced the result.
    pub(crate) id: u64,
    /// Data set that was searched.
    pub(crate) mode: SearchMode,
    /// Offsets into the `SearchData` arrays that matched.
    pub(crate) indices: Vec<usize>,
    /// Scores associated with each index.
    pub(crate) scores: Vec<u16>,
    #[allow(dead_code)]
    /// Whether the worker streamed the complete result set.
    pub(crate) complete: bool,
}
