use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

// Indexing work intentionally runs under strict per-tick limits so UI rendering stays
// responsive even when large trees are being ingested. `MAX_INDEX_UPDATES_PER_TICK`
// bounds how many incremental updates we merge in a single frame, while
// `MAX_INDEX_PROCESSING_TIME` caps the wall-clock time spent applying updates before we
// yield back to drawing and input handling.

use crate::systems::filesystem::{IndexUpdate, merge_update};
use frz_plugin_api::SearchData;

use super::App;
use frz_tui::components::IndexProgress;

impl<'a> App<'a> {
    const MAX_INDEX_UPDATES_PER_TICK: usize = 32;
    const MAX_INDEX_PROCESSING_TIME: Duration = Duration::from_millis(8);

    pub(crate) fn set_index_updates(&mut self, updates: Receiver<IndexUpdate>) {
        self.index_updates = Some(updates);
        if self.data.attributes.is_empty() && self.data.files.is_empty() {
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
            let waiting_for_initial = self.filtered_len() == 0;
            if waiting_for_initial {
                self.initial_results_deadline = Some(Instant::now() + Duration::from_millis(250));
            }
            self.request_search_after_index_update();
            if waiting_for_initial {
                self.wait_for_initial_results();
            }
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
                    update.reset || !update.files.is_empty() || !update.attributes.is_empty();
                if update_changed {
                    merge_update(&mut self.data, &update);
                    self.mark_query_dirty();
                    changed = true;
                }
            }
        }

        let progress = update.progress;
        let attributes_key = crate::plugins::builtin::attributes::descriptor().id;
        let files_key = crate::plugins::builtin::files::descriptor().id;
        self.index_progress.record_indexed(&[
            (attributes_key, progress.indexed_attributes),
            (files_key, progress.indexed_files),
        ]);
        self.index_progress.set_totals(&[
            (attributes_key, progress.total_attributes),
            (files_key, progress.total_files),
        ]);
        if progress.complete {
            self.index_progress.mark_complete();
        }

        changed
    }

    fn wait_for_initial_results(&mut self) {
        let Some(deadline) = self.initial_results_deadline else {
            return;
        };

        if Instant::now() >= deadline {
            self.initial_results_deadline = None;
            return;
        }

        while Instant::now() < deadline {
            self.pump_search_results();
            if self.filtered_len() > 0 {
                return;
            }

            if !self.search.is_in_flight() {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            thread::sleep(Duration::from_millis(10));
        }

        self.pump_search_results();
        if Instant::now() >= deadline {
            self.initial_results_deadline = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::filesystem::ProgressSnapshot;
    use frz_plugin_api::{AttributeRow, FileRow, SearchData};
    use std::time::{Duration, Instant};

    fn wait_for_results(app: &mut App) {
        let deadline = Instant::now() + Duration::from_secs(1);
        while app.search.is_in_flight() && Instant::now() < deadline {
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
            attributes: vec![AttributeRow::new("alpha", 1)].into(),
            progress: ProgressSnapshot {
                indexed_attributes: 1,
                indexed_files: 1,
                total_attributes: Some(1),
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
            app.search.has_unapplied_input(),
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
