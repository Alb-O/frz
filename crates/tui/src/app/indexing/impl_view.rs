use frz_core::filesystem_indexer::{IndexUpdate, IndexView, ProgressSnapshot};

use crate::app::App;

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
