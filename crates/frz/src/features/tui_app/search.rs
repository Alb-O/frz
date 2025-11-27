use std::sync::mpsc::TryRecvError;
use std::time::Instant;

use super::App;
use crate::features::filesystem_indexer::IndexUpdate;
use crate::features::search_pipeline::{MatchBatch, SearchResult, SearchView, SearchViewV2};

impl<'a> App<'a> {
	/// Send a search request for the current query text and mode.
	pub(crate) fn request_search(&mut self) {
		self.issue_search();
	}

	/// Schedule a search refresh due to new index data while respecting the
	/// currently running query.
	pub(crate) fn request_search_after_index_update(&mut self) {
		// When background indexing discovers new entries we keep the UI stable
		// unless the user currently has a query edit that hasn't been
		// processed yet. This lets indexing continue without the visible result
		// list jumping around while the user is idle.
		if self.initial_results_deadline.is_some()
			|| self.search.should_refresh_after_index_update()
		{
			self.issue_search();
		}
	}

	/// Propagate an index update to the background search worker.
	pub(crate) fn notify_search_of_update(&self, update: &IndexUpdate) {
		self.search.notify_of_update(update);
	}

	/// Drain any search results waiting on the receiver channel.
	pub(crate) fn pump_search_results(&mut self) {
		loop {
			match self.search.try_recv() {
				Ok(result) => self.handle_search_result(result),
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => break,
			}
		}
	}

	/// Apply a new search result if it corresponds to the most recent query.
	fn handle_search_result(&mut self, result: SearchResult) {
		if !self.search.matches_latest(result.id) {
			return;
		}

		result.dispatch(self);
	}

	fn issue_search(&mut self) {
		let query = self.search_input.text().to_string();
		self.search.issue_search(query);
	}

	pub(crate) fn settle_initial_results(&mut self, has_results: bool) {
		if let Some(deadline) = self.initial_results_deadline
			&& (has_results || Instant::now() >= deadline)
		{
			self.initial_results_deadline = None;
		}
	}
}

impl<'a> SearchView for App<'a> {
	fn replace_matches(&mut self, indices: Vec<usize>, scores: Vec<u16>) {
		self.apply_match_batch(indices, None, scores);
	}

	fn clear_matches(&mut self) {
		self.tab_buffers.filtered.clear();
		self.tab_buffers.scores.clear();
		self.settle_initial_results(false);
		self.ensure_selection();
	}

	fn record_completion(&mut self, complete: bool) {
		self.search.record_result_completion(complete);
	}

	fn as_v2(&mut self) -> Option<&mut dyn SearchViewV2> {
		Some(self)
	}
}

impl<'a> SearchViewV2 for App<'a> {
	fn replace_matches_v2(&mut self, batch: MatchBatch) {
		let MatchBatch {
			indices,
			ids,
			scores,
		} = batch;
		self.apply_match_batch(indices, ids, scores);
	}
}
