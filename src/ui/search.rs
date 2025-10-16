use std::sync::mpsc::TryRecvError;

use super::App;
use crate::systems::filesystem::IndexUpdate;
use crate::systems::search::SearchResult;

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
        if self.search.should_refresh_after_index_update() {
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

        self.ensure_tab_buffers();
        let entry = self.tab_states.entry(result.mode).or_default();
        entry.filtered = result.indices;
        entry.scores = result.scores;

        self.ensure_selection();

        self.search.record_result_completion(result.complete);
    }

    fn issue_search(&mut self) {
        let query = self.search_input.text().to_string();
        let mode = self.mode;
        self.search.issue_search(query, mode);
    }
}
