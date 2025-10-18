use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use super::commands::{SearchCommand, SearchResult};
use crate::plugins::api::{FrzPluginRegistry, PluginQueryContext, SearchData, SearchStream};

use crate::systems::filesystem::merge_update;

/// Launches the background search worker thread and returns communication channels.
pub(crate) fn spawn(
    mut data: SearchData,
    plugins: FrzPluginRegistry,
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
    plugins: &FrzPluginRegistry,
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
    plugins: &FrzPluginRegistry,
    result_tx: &Sender<SearchResult>,
    latest_query_id: &Arc<AtomicU64>,
    command: SearchCommand,
) -> bool {
    match command {
        SearchCommand::Query { id, query, mode } => {
            if let Some(plugin) = plugins.plugin(mode) {
                let stream = SearchStream::new(result_tx, id, mode);
                let context = PluginQueryContext::new(data, latest_query_id.as_ref());
                plugin.stream(&query, stream, context)
            } else {
                true
            }
        }
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
    use std::time::Duration;

    use crate::plugins::api::{
        PluginSelectionContext, SearchData, SearchMode, SearchSelection,
        descriptors::{
            FrzPluginDataset, FrzPluginDescriptor, FrzPluginUiDefinition, TableContext,
            TableDescriptor,
        },
        registry::FrzPlugin,
        search::FileRow,
        search::SearchStream,
    };

    struct DummyDataset;

    impl FrzPluginDataset for DummyDataset {
        fn key(&self) -> &'static str {
            "dummy"
        }

        fn total_count(&self, _data: &SearchData) -> usize {
            0
        }

        fn build_table<'a>(&self, _context: TableContext<'a>) -> TableDescriptor<'a> {
            TableDescriptor::new(Vec::new(), Vec::new(), Vec::new())
        }
    }

    static DATASET: DummyDataset = DummyDataset;

    static DESCRIPTOR: FrzPluginDescriptor = FrzPluginDescriptor {
        id: "dummy",
        ui: FrzPluginUiDefinition {
            tab_label: "Dummy",
            mode_title: "Dummy",
            hint: "",
            table_title: "",
            count_label: "",
        },
        dataset: &DATASET,
    };

    fn mode() -> SearchMode {
        SearchMode::from_descriptor(&DESCRIPTOR)
    }

    #[derive(Clone)]
    struct DummyPlugin;

    impl FrzPlugin for DummyPlugin {
        fn descriptor(&self) -> &'static FrzPluginDescriptor {
            &DESCRIPTOR
        }

        fn stream(
            &self,
            query: &str,
            stream: SearchStream<'_>,
            context: PluginQueryContext<'_>,
        ) -> bool {
            let query = query.to_lowercase();
            let indices: Vec<usize> = context
                .data()
                .files
                .iter()
                .enumerate()
                .filter_map(|(index, file)| {
                    if file.path.to_lowercase().contains(&query) {
                        Some(index)
                    } else {
                        None
                    }
                })
                .collect();
            let scores = vec![u16::MAX; indices.len()];
            stream.send(indices, scores, true)
        }

        fn selection(
            &self,
            context: PluginSelectionContext<'_>,
            index: usize,
        ) -> Option<SearchSelection> {
            context
                .data()
                .files
                .get(index)
                .cloned()
                .map(SearchSelection::File)
        }
    }

    #[test]
    fn shutdown_command_stops_worker() {
        let data = SearchData::default();
        let plugins = FrzPluginRegistry::default();
        let (tx, _rx, latest) = spawn(data, plugins);
        assert_eq!(latest.load(std::sync::atomic::Ordering::Relaxed), 0);
        tx.send(SearchCommand::Shutdown).unwrap();
    }

    #[test]
    fn streaming_plugin_results_are_forwarded() {
        let data = SearchData::new().with_files(vec![
            FileRow::new("src/lib.rs", Vec::<String>::new()),
            FileRow::new("README.md", Vec::<String>::new()),
        ]);

        let mut registry = FrzPluginRegistry::empty();
        registry.register(DummyPlugin).expect("register plugin");

        let (command_tx, result_rx, _) = spawn(data, registry);
        command_tx
            .send(SearchCommand::Query {
                id: 1,
                query: "readme".to_string(),
                mode: mode(),
            })
            .expect("send query");

        let result = result_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("receive search result");

        assert_eq!(result.id, 1);
        assert_eq!(result.mode, mode());
        assert_eq!(result.indices, vec![1]);
        assert_eq!(result.scores, vec![u16::MAX]);
        assert!(result.complete);

        command_tx
            .send(SearchCommand::Shutdown)
            .expect("send shutdown");
    }
}
