use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

// Indexing work intentionally runs under strict per-tick limits so UI rendering stays
// responsive even when large trees are being ingested. `MAX_INDEX_UPDATES_PER_TICK`
// bounds how many incremental updates we merge in a single frame, while
// `MAX_INDEX_PROCESSING_TIME` caps the wall-clock time spent applying updates before we
// yield back to drawing and input handling.
use frz_core::filesystem_indexer::{IndexResult, IndexUpdate, ProgressSnapshot, merge_update};
use frz_core::search_pipeline::FILES_DATASET_KEY;

use crate::app::App;
use crate::components::IndexProgress;

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

	pub(in crate::app::indexing) fn apply_index_update(&mut self, mut update: IndexUpdate) -> bool {
		let mut changed = false;

		match update.cached_data.take() {
			Some(data) => {
				self.data = data;
				self.results.buffers.filtered.clear();
				self.results.buffers.scores.clear();
				self.results.table_state.select(None);
				self.index_progress
					.refresh_from_data(&self.data, self.dataset_totals());
				self.rebuild_row_id_maps();
				self.mark_query_dirty();
				changed = true;
			}
			None => {
				if update.reset {
					self.index_progress = IndexProgress::with_unknown_totals();
					self.results.buffers.filtered.clear();
					self.results.buffers.scores.clear();
					self.results.table_state.select(None);
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

	pub(in crate::app::indexing) fn record_index_progress_update(
		&mut self,
		progress: ProgressSnapshot,
	) {
		self.index_progress
			.record_indexed(&[(FILES_DATASET_KEY, progress.indexed_files)]);
		self.index_progress
			.set_totals(&[(FILES_DATASET_KEY, progress.total_files)]);
		if progress.complete {
			self.index_progress.mark_complete();
		}
	}

	pub(in crate::app::indexing) fn schedule_search_refresh_after_index_update(
		&mut self,
		changed: bool,
	) {
		if !changed {
			return;
		}

		self.request_search_after_index_update();
	}
}
