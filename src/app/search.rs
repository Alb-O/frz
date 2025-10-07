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
        self.rerun_after_index_update = false;
        self.issue_search();
    }

    /// Schedule a search refresh due to new index data while respecting the
    /// currently running query.
    pub(crate) fn request_search_after_index_update(&mut self) {
        if self.search_in_flight {
            self.rerun_after_index_update = true;
        } else {
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
            if self.rerun_after_index_update {
                self.rerun_after_index_update = false;
                self.issue_search();
            }
        }
    }

    fn issue_search(&mut self) {
        self.next_query_id = self.next_query_id.saturating_add(1);
        let id = self.next_query_id;
        self.latest_query_id = Some(id);
        self.search_in_flight = true;
        let query = self.search_input.text().to_string();
        let mode = self.mode;
        self.search_latest_query_id
            .store(id, AtomicOrdering::Release);
        let _ = self
            .search_tx
            .send(SearchCommand::Query { id, query, mode });
    }
}
