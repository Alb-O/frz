use std::sync::mpsc::Sender;

use crate::extensions::api::{DataStream, StreamEnvelope, ViewAction, ViewTarget};

use super::SearchMode;

/// Consumer responsible for applying streamed search updates.
pub trait SearchView {
    /// Replace the rendered matches for the given mode.
    fn replace_matches(&mut self, mode: SearchMode, indices: Vec<usize>, scores: Vec<u16>);

    /// Clear any rendered matches for the given mode.
    fn clear_matches(&mut self, mode: SearchMode);

    /// Observe the completion state of the stream for the given mode.
    fn record_completion(&mut self, mode: SearchMode, complete: bool);
}

pub struct SearchViewTarget;

impl ViewTarget for SearchViewTarget {
    type View<'target> = dyn SearchView + 'target;
}

/// Aggregated search results emitted back to the UI layer.
pub type SearchAction = ViewAction<SearchViewTarget>;
pub type SearchResult = StreamEnvelope<SearchMode, SearchAction>;

/// Handle used by extensions to stream search results back to the UI.
pub struct SearchStream<'a> {
    inner: DataStream<'a, SearchMode, SearchAction>,
}

impl<'a> SearchStream<'a> {
    /// Create a new stream handle used to send updates to the UI thread.
    #[must_use]
    pub fn new(tx: &'a Sender<SearchResult>, id: u64, mode: SearchMode) -> Self {
        Self {
            inner: DataStream::new(tx, id, mode),
        }
    }

    /// Identifier for the active query.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.inner.id()
    }

    /// Active search mode being serviced.
    #[must_use]
    pub fn mode(&self) -> SearchMode {
        *self.inner.kind()
    }

    /// Send a batch of search results to the UI thread.
    pub fn send(&self, indices: Vec<usize>, scores: Vec<u16>, complete: bool) -> bool {
        let mode = self.mode();
        let empty = indices.is_empty() && scores.is_empty();
        self.send_with(
            move |view| {
                if empty {
                    view.clear_matches(mode);
                } else {
                    view.replace_matches(mode, indices, scores);
                }
                view.record_completion(mode, complete);
            },
            complete,
        )
    }

    /// Send a fully prepared action to the UI thread.
    pub fn send_with(
        &self,
        handler: impl for<'view> FnOnce(&'view mut (dyn SearchView + 'view)) + Send + 'static,
        complete: bool,
    ) -> bool {
        self.inner.send(SearchAction::new(handler), complete)
    }
}

impl<'a> Clone for SearchStream<'a> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
