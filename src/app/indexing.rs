use std::sync::mpsc::{Receiver, TryRecvError};

use crate::indexing::{IndexUpdate, merge_update};
use crate::progress::IndexProgress;

use super::App;

impl<'a> App<'a> {
    pub(crate) fn set_index_updates(&mut self, updates: Receiver<IndexUpdate>) {
        self.index_updates = Some(updates);
        self.index_progress = IndexProgress::with_unknown_totals();
        self.index_progress
            .record_indexed(self.data.facets.len(), self.data.files.len());
    }

    pub(crate) fn pump_index_updates(&mut self) {
        let Some(rx) = self.index_updates.take() else {
            return;
        };

        let mut should_request = false;
        let mut keep_receiver = true;
        loop {
            match rx.try_recv() {
                Ok(update) => {
                    self.notify_search_of_update(&update);
                    should_request |= self.apply_index_update(update);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    keep_receiver = false;
                    break;
                }
            }
        }

        if keep_receiver {
            self.index_updates = Some(rx);
        }

        if should_request {
            self.request_search();
        }
    }

    fn apply_index_update(&mut self, update: IndexUpdate) -> bool {
        let changed = !update.files.is_empty() || !update.facets.is_empty();
        if changed {
            merge_update(&mut self.data, &update);
        }

        let progress = update.progress;
        self.index_progress
            .record_indexed(progress.indexed_facets, progress.indexed_files);
        self.index_progress
            .set_totals(progress.total_facets, progress.total_files);
        if progress.complete {
            self.index_progress.mark_complete();
        }

        changed
    }
}
