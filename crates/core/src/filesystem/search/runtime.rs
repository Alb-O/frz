//! Background search worker thread and command infrastructure.

use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use frz_stream::StreamAction;

use super::{SearchData, SearchResult, SearchStream, stream_files};

/// Commands understood by the background search worker.
#[derive(Debug)]
pub enum SearchCommand {
	/// Run a fuzzy search for the provided query.
	Query {
		/// Identifier that allows the UI to correlate responses with the originating query.
		id: u64,
		/// User supplied query string.
		query: String,
	},
	/// Merge a fresh index update into the existing in-memory search data.
	Update(StreamAction<SearchData>),
	/// Stop the background worker thread.
	Shutdown,
}

/// Launches the background search worker thread and returns communication channels.
pub fn spawn(
	mut data: SearchData,
) -> (
	Sender<SearchCommand>,
	Receiver<SearchResult>,
	Arc<AtomicU64>,
) {
	let (command_tx, command_rx) = mpsc::channel();
	let (result_tx, result_rx) = mpsc::channel();
	let latest_query_id = Arc::new(AtomicU64::new(0));
	let thread_latest = Arc::clone(&latest_query_id);

	thread::spawn(move || worker_loop(&mut data, command_rx, result_tx, thread_latest));

	(command_tx, result_rx, latest_query_id)
}

fn worker_loop(
	data: &mut SearchData,
	command_rx: Receiver<SearchCommand>,
	result_tx: Sender<SearchResult>,
	latest_query_id: Arc<AtomicU64>,
) {
	while let Ok(command) = command_rx.recv() {
		if !handle_command(data, &result_tx, &latest_query_id, command) {
			break;
		}
	}
}

fn handle_command(
	data: &mut SearchData,
	result_tx: &Sender<SearchResult>,
	latest_query_id: &Arc<AtomicU64>,
	command: SearchCommand,
) -> bool {
	match command {
		SearchCommand::Query { id, query } => {
			let stream = SearchStream::new(result_tx, id);
			stream_files(data, &query, stream, latest_query_id)
		}
		SearchCommand::Update(action) => {
			action.apply(data);
			true
		}
		SearchCommand::Shutdown => false,
	}
}
