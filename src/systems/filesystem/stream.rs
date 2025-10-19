use std::sync::mpsc::Sender;

use crate::extensions::api::{DataStream, StreamEnvelope, ViewAction, ViewTarget};

use super::{IndexUpdate, ProgressSnapshot};

/// Metadata describing the type of indexing message being delivered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexKind {
    /// Message derived from cached preview data.
    Preview,
    /// Incremental update emitted while walking the filesystem.
    Update,
    /// Message containing only progress information.
    Progress,
}

/// Consumer that can apply indexing actions emitted from the filesystem worker.
pub trait IndexView {
    /// Forward the raw update to the search subsystem.
    fn forward_index_update(&self, update: &IndexUpdate);

    /// Apply the update to the local search data and return whether anything changed.
    fn apply_index_update(&mut self, update: IndexUpdate) -> bool;

    /// Update progress indicators in the UI.
    fn record_index_progress(&mut self, progress: ProgressSnapshot);

    /// Schedule any follow-up work needed after ingesting an update.
    fn schedule_search_refresh_after_index_update(&mut self, changed: bool);
}

pub struct IndexViewTarget;

impl ViewTarget for IndexViewTarget {
    type View<'target> = dyn IndexView + 'target;
}

/// Result emitted by the filesystem worker.
pub type IndexAction = ViewAction<IndexViewTarget>;
pub type IndexResult = StreamEnvelope<IndexKind, IndexAction>;

/// Handle used to send indexing actions to the UI thread.
pub struct IndexStream<'a> {
    inner: DataStream<'a, IndexKind, IndexAction>,
}

impl<'a> IndexStream<'a> {
    /// Create a new stream handle backed by the provided sender.
    #[must_use]
    pub fn new(tx: &'a Sender<IndexResult>, id: u64, kind: IndexKind) -> Self {
        Self {
            inner: DataStream::new(tx, id, kind),
        }
    }

    /// Emit a fully prepared action with the associated completion state.
    pub fn send_with(
        &self,
        handler: impl for<'view> FnOnce(&'view mut (dyn IndexView + 'view)) + Send + 'static,
        complete: bool,
    ) -> bool {
        self.inner.send(IndexAction::new(handler), complete)
    }

    /// Emit an update describing newly indexed data.
    pub fn send_update(&self, update: IndexUpdate, complete: bool) -> bool {
        let progress = update.progress;
        self.send_with(
            move |view| {
                let payload = update;
                view.forward_index_update(&payload);
                let changed = view.apply_index_update(payload);
                view.record_index_progress(progress);
                view.schedule_search_refresh_after_index_update(changed);
            },
            complete,
        )
    }

    /// Emit a progress-only update without touching the search data.
    pub fn send_progress(&self, progress: ProgressSnapshot, complete: bool) -> bool {
        self.send_with(
            move |view| {
                view.record_index_progress(progress);
                if complete {
                    view.schedule_search_refresh_after_index_update(false);
                }
            },
            complete,
        )
    }
}

impl<'a> Clone for IndexStream<'a> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
