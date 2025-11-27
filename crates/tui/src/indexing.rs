use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

// Indexing work intentionally runs under strict per-tick limits so UI rendering stays
// responsive even when large trees are being ingested. `MAX_INDEX_UPDATES_PER_TICK`
// bounds how many incremental updates we merge in a single frame, while
// `MAX_INDEX_PROCESSING_TIME` caps the wall-clock time spent applying updates before we
// yield back to drawing and input handling.
use frz_core::filesystem_indexer::{
	IndexResult, IndexUpdate, IndexView, ProgressSnapshot, merge_update,
};
use frz_core::search_pipeline::FILES_DATASET_KEY;

use super::App;
use super::components::IndexProgress;

impl<'a> App<'a> {
	const MAX_INDEX_UPDATES_PER_TICK: usize = 32;
	const MAX_INDEX_PROCESSING_TIME: Duration = Duration::from_millis(8);

	pub(crate) fn set_index_updates(&mut self, updates: Receiver<IndexResult>) {
		self.index_updates = Some(updates);
		if self.data.files.is_empty() {
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
				Ok(result) => {
					result.dispatch(self);
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
	}

	fn apply_index_update(&mut self, mut update: IndexUpdate) -> bool {
		let mut changed = false;

		match update.cached_data.take() {
			Some(data) => {
				self.data = data;
				self.tab_buffers.filtered.clear();
				self.tab_buffers.scores.clear();
				self.table_state.select(None);
				self.index_progress
					.refresh_from_data(&self.data, self.dataset_totals());
				self.rebuild_row_id_maps();
				self.mark_query_dirty();
				changed = true;
			}
			None => {
				if update.reset {
					self.index_progress = IndexProgress::with_unknown_totals();
					self.tab_buffers.filtered.clear();
					self.tab_buffers.scores.clear();
					self.table_state.select(None);
				}

				let update_changed = update.reset || !update.files.is_empty();
				if update_changed {
					merge_update(&mut self.data, &update);
					self.rebuild_row_id_maps();
					self.mark_query_dirty();
					changed = true;
				}
			}
		}
		changed
	}

	fn record_index_progress_update(&mut self, progress: ProgressSnapshot) {
		self.index_progress
			.record_indexed(&[(FILES_DATASET_KEY, progress.indexed_files)]);
		self.index_progress
			.set_totals(&[(FILES_DATASET_KEY, progress.total_files)]);
		if progress.complete {
			self.index_progress.mark_complete();
		}
	}

	fn schedule_search_refresh_after_index_update(&mut self, changed: bool) {
		if !changed {
			return;
		}

		self.request_search_after_index_update();
	}
}

impl<'a> IndexView for App<'a> {
	fn forward_index_update(&self, update: &IndexUpdate) {
		self.notify_search_of_update(update);
	}

	fn apply_index_update(&mut self, update: IndexUpdate) -> bool {
		App::apply_index_update(self, update)
	}

	fn record_index_progress(&mut self, progress: ProgressSnapshot) {
		self.record_index_progress_update(progress);
	}

	fn schedule_search_refresh_after_index_update(&mut self, changed: bool) {
		App::schedule_search_refresh_after_index_update(self, changed);
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::time::{Duration, Instant};

	use frz_core::filesystem_indexer::ProgressSnapshot;
	use frz_core::search_pipeline::{FileRow, MatchBatch, SearchData, SearchViewV2};

	use super::*;

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
			files: vec![FileRow::filesystem("src/lib.rs")].into(),
			progress: ProgressSnapshot {
				indexed_files: 1,
				total_files: Some(1),
				complete: true,
			},
			reset: false,
			cached_data: None,
		};

		<App as IndexView>::forward_index_update(&app, &update);
		let progress = update.progress;
		let changed = <App as IndexView>::apply_index_update(&mut app, update);
		<App as IndexView>::record_index_progress(&mut app, progress);
		assert!(changed, "index update should report data changes");
		assert!(
			app.search.has_unapplied_input(),
			"data changes should mark the query dirty"
		);

		<App as IndexView>::schedule_search_refresh_after_index_update(&mut app, changed);
		wait_for_results(&mut app);

		assert!(
			app.filtered_len() > 0,
			"expected refreshed results after indexing update"
		);

		app.mark_query_dirty_from_user_input();
		app.request_search();
		wait_for_results(&mut app);

		assert!(
			app.filtered_len() > 0,
			"expected refreshed results after user input"
		);
	}

	#[test]
	fn row_id_map_tracks_incremental_index_updates() {
		let mut data = SearchData::new();
		let first = FileRow::filesystem("src/lib.rs");
		let second = FileRow::filesystem("src/main.rs");
		data.files = vec![first.clone()];

		let mut app = App::new(data);
		wait_for_results(&mut app);

		let first_id = first.id.expect("expected stable id for first file");
		assert_eq!(
			app.row_id_map.get(&first_id),
			Some(&0),
			"initial row id map should track the first file",
		);

		let second_id = second.id.expect("expected stable id for second file");
		let update = IndexUpdate {
			files: Arc::from(vec![second.clone()]),
			progress: ProgressSnapshot {
				indexed_files: 0,
				total_files: None,
				complete: false,
			},
			reset: false,
			cached_data: None,
		};

		let changed = <App as IndexView>::apply_index_update(&mut app, update);
		assert!(changed, "index update should modify the data set");
		assert_eq!(
			app.row_id_map.get(&first_id),
			Some(&0),
			"first file should keep its index",
		);
		assert_eq!(
			app.row_id_map.get(&second_id),
			Some(&1),
			"second file should be mapped to the appended slot",
		);

		let batch = MatchBatch {
			indices: vec![0],
			ids: Some(vec![second_id]),
			scores: vec![10],
		};
		<App as SearchViewV2>::replace_matches_v2(&mut app, batch);
		let filtered = app.tab_buffers.filtered.clone();
		assert_eq!(
			filtered,
			vec![1],
			"stable ids should resolve to the new index for appended rows",
		);
	}

	#[test]
	fn row_id_map_rebuilds_when_cached_data_applied() {
		let first = FileRow::filesystem("src/lib.rs");
		let second = FileRow::filesystem("src/main.rs");

		let mut data = SearchData::new();
		data.files = vec![first.clone()];

		let mut app = App::new(data);
		wait_for_results(&mut app);

		let cached_data = SearchData {
			context_label: None,
			root: None,
			initial_query: String::new(),
			files: vec![second.clone(), first.clone()],
		};

		let first_id = first.id.expect("expected stable id for first file");
		let second_id = second.id.expect("expected stable id for second file");

		let update = IndexUpdate {
			files: Arc::from(Vec::<FileRow>::new()),
			progress: ProgressSnapshot {
				indexed_files: 0,
				total_files: None,
				complete: false,
			},
			reset: false,
			cached_data: Some(cached_data.clone()),
		};

		let changed = <App as IndexView>::apply_index_update(&mut app, update);
		assert!(changed, "cached data should replace the in-memory dataset");

		assert_eq!(app.row_id_map.get(&second_id), Some(&0));
		assert_eq!(app.row_id_map.get(&first_id), Some(&1));

		let batch = MatchBatch {
			indices: vec![1],
			ids: Some(vec![first_id]),
			scores: vec![5],
		};
		<App as SearchViewV2>::replace_matches_v2(&mut app, batch);
		let filtered = app.tab_buffers.filtered.clone();
		assert_eq!(
			filtered,
			vec![1],
			"stable ids should resolve to indices from the cached dataset",
		);
	}
}
