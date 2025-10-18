fn main() {
    use std::sync::atomic::Ordering;

    use frz::extensions::api::{
        Contribution, ExtensionCatalog, ExtensionModule, ExtensionPackage, ExtensionQueryContext,
        ExtensionSelectionContext, SearchSelection, SearchStream,
        descriptors::{
            ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext,
            TableDescriptor,
        },
    };
    use ratatui::widgets::Row;

    static DATASET: DemoDataset = DemoDataset;
    static DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
        id: "capability-demo",
        ui: ExtensionUiDefinition {
            tab_label: "Capability Demo",
            mode_title: "Capability Demo",
            hint: "Type to search",
            table_title: "Results",
            count_label: "Items",
        },
        dataset: &DATASET,
    };

    struct DemoDataset;

    impl ExtensionDataset for DemoDataset {
        fn key(&self) -> &'static str {
            "capability-demo"
        }

        fn total_count(&self, _data: &frz::extensions::api::SearchData) -> usize {
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

    struct DemoModule;

    impl ExtensionModule for DemoModule {
        fn descriptor(&self) -> &'static ExtensionDescriptor {
            &DESCRIPTOR
        }

        fn stream(
            &self,
            query: &str,
            stream: SearchStream<'_>,
            context: ExtensionQueryContext<'_>,
        ) -> bool {
            let latest_id = context.latest_query_id().load(Ordering::Relaxed);
            println!("received query: {query:?}, latest id: {latest_id}");
            stream.send(Vec::new(), Vec::new(), true)
        }

        fn selection(
            &self,
            _context: ExtensionSelectionContext<'_>,
            _index: usize,
        ) -> Option<SearchSelection> {
            None
        }
    }

    struct DemoPackage;

    impl ExtensionPackage for DemoPackage {
        type Contributions<'a>
            = std::iter::Once<Contribution>
        where
            Self: 'a;

        fn contributions(&self) -> Self::Contributions<'_> {
            std::iter::once(Contribution::search_tab(&DESCRIPTOR, DemoModule))
        }
    }

    let mut catalog = ExtensionCatalog::new();
    catalog
        .register_package(DemoPackage)
        .expect("demo package should register");

    for descriptor in catalog.descriptors() {
        println!("Registered contribution: {}", descriptor.id);
    }
}
