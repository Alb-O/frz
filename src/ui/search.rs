use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::mpsc::TryRecvError;

#[cfg(feature = "fs")]
use crate::indexing::IndexUpdate;
use crate::search::{SearchCommand, SearchResult};
use crate::types::SearchMode;

use super::App;

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
        if !self.search_in_flight
            && self.input_revision != self.last_applied_revision
            && self.input_revision == self.last_user_input_revision
        {
            self.issue_search();
        }
    }

    /// Propagate an index update to the background search worker.
    #[cfg(feature = "fs")]
    pub(crate) fn notify_search_of_update(&self, update: &IndexUpdate) {
        let _ = self.search_tx.send(SearchCommand::Update(update.clone()));
    }

    /// Drain any search results waiting on the receiver channel.
    pub(crate) fn pump_search_results(&mut self) {
        loop {
            match self.search_rx.try_recv() {
                Ok(result) => self.handle_search_result(result),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Apply a new search result if it corresponds to the most recent query.
    fn handle_search_result(&mut self, result: SearchResult) {
        if Some(result.id) != self.latest_query_id {
            return;
        }

        match result.mode {
            SearchMode::Facets => {
                self.filtered_facets = result.indices;
                self.facet_scores = result.scores;
            }
            SearchMode::Files => {
                self.filtered_files = result.indices;
                self.file_scores = result.scores;
            }
        }

        self.ensure_selection();

        if result.complete {
            self.search_in_flight = false;
            self.last_applied_revision = self.pending_result_revision;
        }
    }

    fn issue_search(&mut self) {
        self.next_query_id = self.next_query_id.saturating_add(1);
        let id = self.next_query_id;
        self.latest_query_id = Some(id);
        self.search_in_flight = true;
        self.pending_result_revision = self.input_revision;
        let query = self.search_input.text().to_string();
        let mode = self.mode;
        self.search_latest_query_id
            .store(id, AtomicOrdering::Release);
        let _ = self
            .search_tx
            .send(SearchCommand::Query { id, query, mode });
    }
}
