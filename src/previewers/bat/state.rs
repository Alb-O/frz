use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use super::key::PreviewKey;
use super::worker::render_file;

struct PendingPreview {
    key: PreviewKey,
    receiver: Receiver<Result<String, String>>,
}

struct CachedPreview {
    key: PreviewKey,
    output: Result<String, String>,
}

/// Tracks the state of an asynchronous preview request along with the latest
/// cached output.
///
/// The struct owns both the cached result, if any, and the communication
/// channel to a worker thread that is generating a new preview. It is designed
/// to be wrapped in a [`Mutex`](std::sync::Mutex) so `TextPreviewer` can manage
/// concurrent UI access.
#[derive(Default)]
pub(super) struct PreviewState {
    cached: Option<CachedPreview>,
    pending: Option<PendingPreview>,
}

impl PreviewState {
    /// Polls the pending preview receiver, caching the result if it has
    /// finished.
    pub(super) fn poll_pending(&mut self) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };

        match pending.receiver.try_recv() {
            Ok(result) => {
                let key = pending.key.clone();
                self.cached = Some(CachedPreview {
                    key,
                    output: result,
                });
                self.pending = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
            }
        }
    }

    /// Returns the cached result for the given [`PreviewKey`] if it matches the
    /// most recent preview request.
    pub(super) fn cached_result(&self, key: &PreviewKey) -> Option<Result<String, String>> {
        self.cached
            .as_ref()
            .filter(|cached| cached.key == *key)
            .map(|cached| cached.output.clone())
    }

    /// Returns the cached output regardless of which [`PreviewKey`] produced
    /// it.
    pub(super) fn cached_output(&self) -> Option<Result<String, String>> {
        self.cached.as_ref().map(|cached| cached.output.clone())
    }

    /// Ensures that we have a worker thread rendering the requested preview.
    pub(super) fn ensure_request(&mut self, key: PreviewKey) {
        if let Some(pending) = &self.pending
            && pending.key == key
        {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        let path = key.path.clone();
        let width = key.width;
        let bat_theme = key.bat_theme.clone();
        let git_modifications = key.git_modifications;
        thread::spawn(move || {
            let result = render_file(path, width, bat_theme.as_deref(), git_modifications);
            let _ = sender.send(result);
        });
        self.pending = Some(PendingPreview { key, receiver });
    }
}
