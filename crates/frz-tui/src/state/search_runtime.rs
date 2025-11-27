//! Background coordination for search queries and index updates.
//!
//! The [`SearchRuntime`] encapsulates communication with the asynchronous
//! search worker, ensuring requests are sequenced correctly and that only the
//! newest results influence UI state.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use frz_stream::StreamAction;

use frz_core::features::filesystem_indexer::{IndexUpdate, merge_update};
use frz_core::features::search_pipeline::runtime::SearchCommand;
use frz_core::features::search_pipeline::{SearchData, SearchResult};

/// Tracks the revision counters used to determine when data has changed.
#[derive(Default)]
struct RevisionState {
	input: u64,
	pending_result: u64,
	last_applied: u64,
	last_user_input: u64,
}

/// Thin wrapper around the search worker channels.
pub(crate) struct SearchRuntime {
	tx: Sender<SearchCommand>,
	rx: Receiver<SearchResult>,
	latest_query_id: Arc<AtomicU64>,
	next_query_id: u64,
	current_query_id: Option<u64>,
	in_flight: bool,
	user_has_typed: bool,
	revisions: RevisionState,
}

impl SearchRuntime {
	/// Create a new search runtime with the provided communication channels.
	pub(crate) fn new(
		tx: Sender<SearchCommand>,
		rx: Receiver<SearchResult>,
		latest_query_id: Arc<AtomicU64>,
	) -> Self {
		Self {
			tx,
			rx,
			latest_query_id,
			next_query_id: 0,
			current_query_id: None,
			in_flight: false,
			user_has_typed: false,
			revisions: RevisionState::default(),
		}
	}

	pub(crate) fn shutdown(&self) {
		let _ = self.tx.send(SearchCommand::Shutdown);
	}

	pub(crate) fn mark_query_dirty(&mut self) {
		self.revisions.input = self.revisions.input.wrapping_add(1);
	}

	pub(crate) fn mark_query_dirty_from_user_input(&mut self) {
		self.mark_query_dirty();
		self.user_has_typed = true;
		self.revisions.last_user_input = self.revisions.input;
	}

	pub(crate) fn issue_search(&mut self, query: String) {
		self.next_query_id = self.next_query_id.saturating_add(1);
		let id = self.next_query_id;
		self.current_query_id = Some(id);
		self.in_flight = true;
		self.revisions.pending_result = self.revisions.input;
		self.latest_query_id.store(id, AtomicOrdering::Release);
		let _ = self.tx.send(SearchCommand::Query { id, query });
	}

	pub(crate) fn should_refresh_after_index_update(&self) -> bool {
		!self.in_flight
			&& self.revisions.input != self.revisions.last_applied
			&& self.revisions.input == self.revisions.last_user_input
	}

	pub(crate) fn matches_latest(&self, result_id: u64) -> bool {
		Some(result_id) == self.current_query_id
	}

	pub(crate) fn record_result_completion(&mut self, complete: bool) {
		if complete {
			self.in_flight = false;
			self.revisions.last_applied = self.revisions.pending_result;
			self.revisions.last_user_input = self.revisions.last_applied;
		}
	}

	pub(crate) fn has_issued_query(&self) -> bool {
		self.current_query_id.is_some()
	}

	pub(crate) fn user_has_typed(&self) -> bool {
		self.user_has_typed
	}

	#[cfg(test)]
	pub(crate) fn is_in_flight(&self) -> bool {
		self.in_flight
	}

	#[cfg(test)]
	pub(crate) fn has_unapplied_input(&self) -> bool {
		self.revisions.input != self.revisions.last_applied
	}

	pub(crate) fn try_recv(&mut self) -> Result<SearchResult, TryRecvError> {
		self.rx.try_recv()
	}

	pub(crate) fn notify_of_update(&self, update: &IndexUpdate) {
		let action = StreamAction::new({
			let update = update.clone();
			move |data: &mut SearchData| {
				merge_update(data, &update);
			}
		});
		let _ = self.tx.send(SearchCommand::Update(action));
	}
}

#[cfg(test)]
mod tests {
	use std::sync::mpsc;

	use super::*;
	use frz_core::features::search_pipeline::runtime::SearchCommand;

	#[test]
	fn partial_completion_does_not_finalize_query() {
		let (command_tx, _command_rx) = mpsc::channel::<SearchCommand>();
		let (_result_tx, result_rx) = mpsc::channel();
		let latest = Arc::new(AtomicU64::new(0));
		let mut runtime = SearchRuntime::new(command_tx, result_rx, Arc::clone(&latest));

		runtime.mark_query_dirty();
		runtime.issue_search("example".into());
		assert!(runtime.is_in_flight());
		assert!(runtime.has_unapplied_input());

		runtime.record_result_completion(false);
		assert!(runtime.is_in_flight());
		assert!(runtime.has_unapplied_input());

		runtime.record_result_completion(true);
		assert!(!runtime.is_in_flight());
		assert!(!runtime.has_unapplied_input());
	}
}
