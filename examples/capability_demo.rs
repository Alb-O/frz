fn main() {
    use std::sync::atomic::Ordering;

    use frz_plugin_api::{
        Capability, PluginBundle, PluginQueryContext, PluginSelectionContext, SearchPlugin,
        SearchPluginRegistry, SearchSelection, SearchStream,
        descriptors::{
            SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
            TableDescriptor,
        },
    };
    use ratatui::widgets::Row;

    static DATASET: DemoDataset = DemoDataset;
    static DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
        id: "capability-demo",
        ui: SearchPluginUiDefinition {
            tab_label: "Capability Demo",
            mode_title: "Capability Demo",
            hint: "Type to search",
            table_title: "Results",
            count_label: "Items",
        },
        dataset: &DATASET,
    };

    struct DemoDataset;

    impl SearchPluginDataset for DemoDataset {
        fn key(&self) -> &'static str {
            "capability-demo"
        }

        fn total_count(&self, _data: &frz_plugin_api::SearchData) -> usize {
            0
        }

        fn build_table<'a>(&self, _context: TableContext<'a>) -> TableDescriptor<'a> {
            TableDescriptor::new(
                vec!["Sample".to_string()],
                Vec::new(),
                vec![Row::new(vec!["Enable the worker to stream results"])],
            )
        }
    }

    struct DemoPlugin;

    impl SearchPlugin for DemoPlugin {
        fn descriptor(&self) -> &'static SearchPluginDescriptor {
            &DESCRIPTOR
        }

        fn stream(
            &self,
            query: &str,
            stream: SearchStream<'_>,
            context: PluginQueryContext<'_>,
        ) -> bool {
            let latest_id = context.latest_query_id().load(Ordering::Relaxed);
            println!("received query: {query:?}, latest id: {latest_id}");
            stream.send(Vec::new(), Vec::new(), true)
        }

        fn selection(
            &self,
            _context: PluginSelectionContext<'_>,
            _index: usize,
        ) -> Option<SearchSelection> {
            None
        }
    }

    struct DemoBundle;

    impl PluginBundle for DemoBundle {
        type Capabilities<'a>
            = std::iter::Once<Capability>
        where
            Self: 'a;

        fn capabilities(&self) -> Self::Capabilities<'_> {
            std::iter::once(Capability::search_tab(&DESCRIPTOR, DemoPlugin))
        }
    }

    let mut registry = SearchPluginRegistry::new();
    registry
        .register_bundle(DemoBundle)
        .expect("demo bundle should register");

    for descriptor in registry.descriptors() {
        println!("Registered capability: {}", descriptor.id);
    }
}
