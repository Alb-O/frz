use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use super::commands::{SearchCommand, SearchResult};
use super::streaming::{stream_facets, stream_files};
use crate::types::{SearchData, SearchMode};

#[cfg(feature = "fs")]
use crate::indexing::merge_update;

/// Launches the background search worker thread and returns communication channels.
#[cfg_attr(not(feature = "fs"), allow(unused_mut))]
pub(crate) fn spawn(
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
        SearchCommand::Query { id, query, mode } => match mode {
            SearchMode::Facets => {
                stream_facets(data, &query, id, result_tx, latest_query_id.as_ref())
            }
            SearchMode::Files => {
                stream_files(data, &query, id, result_tx, latest_query_id.as_ref())
            }
        },
        #[cfg(feature = "fs")]
        SearchCommand::Update(update) => {
            merge_update(data, &update);
            true
        }
        SearchCommand::Shutdown => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SearchData;

    #[test]
    fn shutdown_command_stops_worker() {
        let data = SearchData::default();
        let (tx, _rx, latest) = spawn(data);
        assert_eq!(latest.load(std::sync::atomic::Ordering::Relaxed), 0);
        tx.send(SearchCommand::Shutdown).unwrap();
    }
}
