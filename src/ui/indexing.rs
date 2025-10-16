use std::sync::mpsc::Receiver;

#[cfg(feature = "fs")]
use std::time::{Duration, Instant};

#[cfg(feature = "fs")]
use std::sync::mpsc::TryRecvError;

#[cfg(feature = "fs")]
use crate::systems::filesystem::IndexUpdate;
#[cfg(feature = "fs")]
use crate::systems::filesystem::merge_update;
#[cfg(not(feature = "fs"))]
type IndexUpdate = ();
#[cfg(feature = "fs")]
use frz_plugin_api::SearchData;

use super::App;
#[cfg(feature = "fs")]
use super::components::progress::IndexProgress;

#[cfg(feature = "fs")]
impl<'a> App<'a> {
    const MAX_INDEX_UPDATES_PER_TICK: usize = 32;
    const MAX_INDEX_PROCESSING_TIME: Duration = Duration::from_millis(8);

    pub(crate) fn set_index_updates(&mut self, updates: Receiver<IndexUpdate>) {
        self.index_updates = Some(updates);
        if self.data.facets.is_empty() && self.data.files.is_empty() {
            self.index_progress = IndexProgress::with_unknown_totals();
        } else {
            self.index_progress
                .refresh_from_data(&self.data, self.dataset_totals());
        }
    }

    pub(crate) fn pump_index_updates(&mut self) {
        let Some(rx) = self.index_updates.take() else {
            return;
        };

        let mut should_request = false;
        let mut keep_receiver = true;
        let mut processed = 0usize;
        let start = Instant::now();

        loop {
            if processed >= Self::MAX_INDEX_UPDATES_PER_TICK
                || start.elapsed() >= Self::MAX_INDEX_PROCESSING_TIME
            {
                break;
            }
            match rx.try_recv() {
                Ok(mut update) => {
                    let cached_data = update.cached_data.take();
                    self.notify_search_of_update(&update);
                    should_request |= self.apply_index_update(update, cached_data);
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

    fn apply_index_update(&mut self, update: IndexUpdate, cached_data: Option<SearchData>) -> bool {
        let mut changed = false;

        match cached_data {
            Some(data) => {
                self.data = data;
                for state in self.tab_states.values_mut() {
                    state.filtered.clear();
                    state.scores.clear();
                }
                self.table_state.select(None);
                self.index_progress
                    .refresh_from_data(&self.data, self.dataset_totals());
                self.mark_query_dirty();
                changed = true;
            }
            None => {
                if update.reset {
                    self.index_progress = IndexProgress::with_unknown_totals();
                    for state in self.tab_states.values_mut() {
                        state.filtered.clear();
                        state.scores.clear();
                    }
                    self.table_state.select(None);
                }

                let update_changed =
                    update.reset || !update.files.is_empty() || !update.facets.is_empty();
                if update_changed {
                    merge_update(&mut self.data, &update);
                    self.mark_query_dirty();
                    changed = true;
                }
            }
        }

        let progress = update.progress;
        let facets_key = crate::plugins::builtin::facets::descriptor().id;
        let files_key = crate::plugins::builtin::files::descriptor().id;
        self.index_progress.record_indexed(&[
            (facets_key, progress.indexed_facets),
            (files_key, progress.indexed_files),
        ]);
        self.index_progress.set_totals(&[
            (facets_key, progress.total_facets),
            (files_key, progress.total_files),
        ]);
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
    use crate::systems::filesystem::ProgressSnapshot;
    use frz_plugin_api::{FacetRow, FileRow, SearchData};
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
            files: vec![FileRow::filesystem("src/lib.rs", ["alpha"])].into(),
            facets: vec![FacetRow::new("alpha", 1)].into(),
            progress: ProgressSnapshot {
                indexed_facets: 1,
                indexed_files: 1,
                total_facets: Some(1),
                total_files: Some(1),
                complete: true,
            },
            reset: false,
            cached_data: None,
        };

        app.notify_search_of_update(&update);
        let changed = app.apply_index_update(update, None);
        assert!(changed, "index update should report data changes");
        assert!(
            app.input_revision != app.last_applied_revision,
            "data changes should mark the query dirty"
        );

        app.request_search_after_index_update();
        wait_for_results(&mut app);

        assert_eq!(
            app.filtered_len(),
            0,
            "results should remain stable until the user edits the query"
        );

        app.mark_query_dirty_from_user_input();
        app.request_search();
        wait_for_results(&mut app);

        assert!(
            app.filtered_len() > 0,
            "expected refreshed results after user input"
        );
    }
}
