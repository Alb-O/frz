use std::sync::mpsc::Sender;

use super::SearchMode;

/// Aggregated search results emitted back to the UI layer.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Identifier matching the query that produced the result.
    pub id: u64,
    /// Data set that was searched.
    pub mode: SearchMode,
    /// Offsets into the `SearchData` arrays that matched.
    pub indices: Vec<usize>,
    /// Scores associated with each index.
    pub scores: Vec<u16>,
    /// Whether the worker streamed the complete result set.
    pub complete: bool,
}

/// Handle used by extensions to stream search results back to the UI.
pub struct SearchStream<'a> {
    tx: &'a Sender<SearchResult>,
    id: u64,
    mode: SearchMode,
}

impl<'a> SearchStream<'a> {
    /// Create a new stream handle used to send updates to the UI thread.
    #[must_use]
    pub fn new(tx: &'a Sender<SearchResult>, id: u64, mode: SearchMode) -> Self {
        Self { tx, id, mode }
    }

    /// Identifier for the active query.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Active search mode being serviced.
    #[must_use]
    pub fn mode(&self) -> SearchMode {
        self.mode
    }

    /// Send a batch of search results to the UI thread.
    pub fn send(&self, indices: Vec<usize>, scores: Vec<u16>, complete: bool) -> bool {
        self.tx
            .send(SearchResult {
                id: self.id,
                mode: self.mode,
                indices,
                scores,
                complete,
            })
            .is_ok()
    }
}

impl<'a> Clone for SearchStream<'a> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx,
            id: self.id,
            mode: self.mode,
        }
    }
}
