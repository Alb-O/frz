use std::sync::mpsc::Sender;

use crate::extensions::api::{DataStream, StreamEnvelope, ViewAction, ViewTarget};

use super::SearchMode;

/// Batch of search matches emitted by a producer.
#[derive(Clone)]
pub struct MatchBatch {
    pub indices: Vec<usize>,
    pub ids: Option<Vec<u64>>,
    pub scores: Vec<u16>,
}

impl MatchBatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let ids_empty = self.ids.as_ref().is_none_or(|ids| ids.is_empty());
        self.indices.is_empty() && self.scores.is_empty() && ids_empty
    }
}

/// Consumer responsible for applying streamed search updates.
pub trait SearchView {
    /// Replace the rendered matches for the given mode.
    fn replace_matches(&mut self, mode: SearchMode, indices: Vec<usize>, scores: Vec<u16>);

    /// Clear any rendered matches for the given mode.
    fn clear_matches(&mut self, mode: SearchMode);

    /// Observe the completion state of the stream for the given mode.
    ///
    /// The `complete` flag is `true` exactly once per query and signals that no
    /// further updates will arrive for the associated [`SearchStream::id`].
    /// Consumers should use this to retire in-flight progress indicators or
    /// trigger follow-up work that depends on the final result set. Partial
    /// flushes set `complete` to `false` to indicate that additional batches are
    /// pending.
    fn record_completion(&mut self, mode: SearchMode, complete: bool);

    /// Attempt to upgrade to the V2 search view if supported.
    fn as_v2(&mut self) -> Option<&mut dyn SearchViewV2> {
        None
    }
}

/// Optional extension for [`SearchView`] implementors that understand stable
/// row identifiers.
pub trait SearchViewV2 {
    fn replace_matches_v2(&mut self, mode: SearchMode, batch: MatchBatch);
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
    ///
    /// The `complete` flag matches the value passed to [`record_completion`] on
    /// the receiving [`SearchView`], allowing the consumer to distinguish
    /// between partial flushes and the terminal update for a query.
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

    /// Send a batch of search results to the UI thread using the new
    /// identifier-aware path when available.
    pub fn send_batch(&self, batch: MatchBatch, complete: bool) -> bool {
        let mode = self.mode();
        let empty = batch.is_empty();
        self.send_with(
            move |view| {
                if empty {
                    view.clear_matches(mode);
                } else if let Some(view2) = view.as_v2() {
                    view2.replace_matches_v2(mode, batch);
                } else {
                    let MatchBatch {
                        indices, scores, ..
                    } = batch;
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
