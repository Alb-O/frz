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
    ///
    /// A `complete` flag of `true` indicates that the filesystem worker has
    /// finished streaming updates for the current crawl pass.
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
    ///
    /// When `complete` is `true`, consumers receive a final notification via
    /// [`IndexView::schedule_search_refresh_after_index_update`] so they can
    /// reconcile any pending search refresh with the finished crawl.
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, mpsc};

    use crate::systems::filesystem::IndexUpdate;

    #[derive(Default)]
    struct RecordingView {
        scheduled: Vec<bool>,
    }

    impl IndexView for RecordingView {
        fn forward_index_update(&self, _update: &IndexUpdate) {}

        fn apply_index_update(&mut self, _update: IndexUpdate) -> bool {
            false
        }

        fn record_index_progress(&mut self, _progress: ProgressSnapshot) {}

        fn schedule_search_refresh_after_index_update(&mut self, changed: bool) {
            self.scheduled.push(changed);
        }
    }

    #[test]
    fn update_envelope_carries_completion_flag() {
        let (tx, rx) = mpsc::channel();
        let stream = IndexStream::new(&tx, 1, IndexKind::Update);
        let update = IndexUpdate {
            files: Arc::new([]),
            attributes: Arc::new([]),
            progress: ProgressSnapshot {
                indexed_attributes: 0,
                indexed_files: 0,
                total_attributes: None,
                total_files: None,
                complete: false,
            },
            reset: false,
            cached_data: None,
        };
        assert!(stream.send_update(update.clone(), false));
        let first = rx.recv().expect("first envelope");
        assert!(!first.complete);
        assert!(stream.send_update(update, true));
        let second = rx.recv().expect("second envelope");
        assert!(second.complete);
    }

    #[test]
    fn progress_completion_triggers_schedule() {
        let (tx, rx) = mpsc::channel();
        let stream = IndexStream::new(&tx, 2, IndexKind::Progress);
        let progress = ProgressSnapshot {
            indexed_attributes: 0,
            indexed_files: 0,
            total_attributes: None,
            total_files: None,
            complete: false,
        };

        assert!(stream.send_progress(progress, false));
        assert!(stream.send_progress(
            ProgressSnapshot {
                complete: true,
                ..progress
            },
            true
        ));

        let mut view = RecordingView::default();
        let first = rx.recv().expect("first envelope");
        assert!(!first.complete);
        first.dispatch(&mut view);
        assert!(view.scheduled.is_empty());

        let second = rx.recv().expect("second envelope");
        assert!(second.complete);
        second.dispatch(&mut view);
        assert_eq!(view.scheduled, vec![false]);
    }
}
