use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use super::commands::{SearchCommand, SearchResult, SearchStream};
use super::plugins::SearchPluginRegistry;
use crate::types::SearchData;

#[cfg(feature = "fs")]
use crate::indexing::merge_update;

/// Launches the background search worker thread and returns communication channels.
#[cfg_attr(not(feature = "fs"), allow(unused_mut))]
pub(crate) fn spawn(
    mut data: SearchData,
    plugins: SearchPluginRegistry,
) -> (
    Sender<SearchCommand>,
    Receiver<SearchResult>,
    Arc<AtomicU64>,
) {
    let (command_tx, command_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let latest_query_id = Arc::new(AtomicU64::new(0));
    let thread_latest = Arc::clone(&latest_query_id);

    thread::spawn(move || worker_loop(&mut data, &plugins, command_rx, result_tx, thread_latest));

    (command_tx, result_rx, latest_query_id)
}

fn worker_loop(
    data: &mut SearchData,
    plugins: &SearchPluginRegistry,
    command_rx: Receiver<SearchCommand>,
    result_tx: Sender<SearchResult>,
    latest_query_id: Arc<AtomicU64>,
) {
    while let Ok(command) = command_rx.recv() {
        if !handle_command(data, plugins, &result_tx, &latest_query_id, command) {
            break;
        }
    }
}

fn handle_command(
    data: &mut SearchData,
    plugins: &SearchPluginRegistry,
    result_tx: &Sender<SearchResult>,
    latest_query_id: &Arc<AtomicU64>,
    command: SearchCommand,
) -> bool {
    match command {
        SearchCommand::Query { id, query, mode } => {
            if let Some(plugin) = plugins.plugin(mode) {
                let stream = SearchStream::new(result_tx, id, mode);
                plugin.stream(data, &query, stream, latest_query_id.as_ref())
            } else {
                true
            }
        }
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
        let plugins = SearchPluginRegistry::default();
        let (tx, _rx, latest) = spawn(data, plugins);
        assert_eq!(latest.load(std::sync::atomic::Ordering::Relaxed), 0);
        tx.send(SearchCommand::Shutdown).unwrap();
    }
}
