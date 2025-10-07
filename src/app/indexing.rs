use std::sync::mpsc::Receiver;

#[cfg(feature = "fs")]
use std::sync::mpsc::TryRecvError;

use crate::indexing::IndexUpdate;
#[cfg(feature = "fs")]
use crate::indexing::merge_update;
#[cfg(feature = "fs")]
use crate::progress::IndexProgress;

use super::App;

#[cfg(feature = "fs")]
impl<'a> App<'a> {
    const MAX_INDEX_UPDATES_PER_TICK: usize = 32;

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
        let mut processed = 0usize;
        loop {
            if processed >= Self::MAX_INDEX_UPDATES_PER_TICK {
                break;
            }
            match rx.try_recv() {
                Ok(update) => {
                    self.notify_search_of_update(&update);
                    should_request |= self.apply_index_update(update);
                    processed += 1;
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
            self.request_search_after_index_update();
        }
    }

    fn apply_index_update(&mut self, update: IndexUpdate) -> bool {
        let changed = !update.files.is_empty() || !update.facets.is_empty();
        if changed {
            merge_update(&mut self.data, &update);
            self.mark_query_dirty();
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

#[cfg(not(feature = "fs"))]
impl<'a> App<'a> {
    #[allow(dead_code)]
    pub(crate) fn set_index_updates(&mut self, _updates: Receiver<IndexUpdate>) {
        let _ = _updates;
    }

    pub(crate) fn pump_index_updates(&mut self) {}
}

#[cfg(all(test, feature = "fs"))]
mod tests {
    use super::*;
    use crate::indexing::ProgressSnapshot;
    use crate::types::{FacetRow, FileRow, SearchData};
    use std::time::{Duration, Instant};

    fn wait_for_results(app: &mut App) {
        let deadline = Instant::now() + Duration::from_secs(1);
        while app.search_in_flight && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
            app.pump_search_results();
        }
        app.pump_search_results();
    }

    #[test]
    fn index_updates_refresh_results_without_input_changes() {
        let data = SearchData::new();
        let mut app = App::new(data);

        app.mark_query_dirty();
        app.request_search();
        wait_for_results(&mut app);

        assert_eq!(app.filtered_len(), 0);
        let update = IndexUpdate {
            files: vec![FileRow::new("src/lib.rs", ["alpha"])].into(),
            facets: vec![FacetRow::new("alpha", 1)].into(),
            progress: ProgressSnapshot {
                indexed_facets: 1,
                indexed_files: 1,
                total_facets: Some(1),
                total_files: Some(1),
                complete: true,
            },
        };

        app.notify_search_of_update(&update);
        let changed = app.apply_index_update(update);
        assert!(changed, "index update should report data changes");
        assert!(
            app.input_revision != app.last_applied_revision,
            "data changes should mark the query dirty"
        );

        app.request_search_after_index_update();
        wait_for_results(&mut app);

        assert!(
            app.filtered_len() > 0,
            "expected refreshed results after indexing"
        );
    }
}
