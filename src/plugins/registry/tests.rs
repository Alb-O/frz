use super::*;
use crate::plugins::{
    PluginQueryContext, PluginSelectionContext,
    descriptors::{
        SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
        TableDescriptor,
    },
    systems::search::SearchStream,
};
use crate::types::{SearchData, SearchMode, SearchSelection};

struct TestDataset;

impl SearchPluginDataset for TestDataset {
    fn key(&self) -> &'static str {
        "test"
    }

    fn total_count(&self, _data: &SearchData) -> usize {
        0
    }

    fn build_table<'a>(&self, _context: TableContext<'a>) -> TableDescriptor<'a> {
        TableDescriptor::new(Vec::new(), Vec::new(), Vec::new())
    }
}

static TEST_DATASET: TestDataset = TestDataset;

static TEST_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
    id: "test",
    ui: SearchPluginUiDefinition {
        tab_label: "Test",
        mode_title: "Test Mode",
        hint: "",
        table_title: "",
        count_label: "",
    },
    dataset: &TEST_DATASET,
};

fn test_mode() -> SearchMode {
    SearchMode::from_descriptor(&TEST_DESCRIPTOR)
}

struct AlternateDataset;

impl SearchPluginDataset for AlternateDataset {
    fn key(&self) -> &'static str {
        "alt"
    }

    fn total_count(&self, _data: &SearchData) -> usize {
        0
    }

    fn build_table<'a>(&self, _context: TableContext<'a>) -> TableDescriptor<'a> {
        TableDescriptor::new(Vec::new(), Vec::new(), Vec::new())
    }
}

static ALT_DATASET: AlternateDataset = AlternateDataset;

static ALT_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
    id: "alt",
    ui: SearchPluginUiDefinition {
        tab_label: "Alt",
        mode_title: "Alt Mode",
        hint: "",
        table_title: "",
        count_label: "",
    },
    dataset: &ALT_DATASET,
};

fn alt_mode() -> SearchMode {
    SearchMode::from_descriptor(&ALT_DESCRIPTOR)
}

struct TestPlugin;

impl SearchPlugin for TestPlugin {
    fn descriptor(&self) -> &'static SearchPluginDescriptor {
        &TEST_DESCRIPTOR
    }

    fn stream(
        &self,
        _query: &str,
        _stream: SearchStream<'_>,
        _context: PluginQueryContext<'_>,
    ) -> bool {
        false
    }

    fn selection(
        &self,
        _context: PluginSelectionContext<'_>,
        _index: usize,
    ) -> Option<SearchSelection> {
        None
    }
}

struct AlternatePlugin;

impl SearchPlugin for AlternatePlugin {
    fn descriptor(&self) -> &'static SearchPluginDescriptor {
        &ALT_DESCRIPTOR
    }

    fn stream(
        &self,
        _query: &str,
        _stream: SearchStream<'_>,
        _context: PluginQueryContext<'_>,
    ) -> bool {
        false
    }

    fn selection(
        &self,
        _context: PluginSelectionContext<'_>,
        _index: usize,
    ) -> Option<SearchSelection> {
        None
    }
}

#[test]
fn registers_plugins_in_insertion_order() {
    let mut registry = SearchPluginRegistry::empty();
    registry.register(TestPlugin);
    registry.register(AlternatePlugin);
    let modes: Vec<SearchMode> = registry.iter().map(|plugin| plugin.mode()).collect();
    assert_eq!(modes, vec![test_mode(), alt_mode()]);
}

#[test]
fn deregister_removes_plugin_and_updates_indexes() {
    let mut registry = SearchPluginRegistry::empty();
    registry.register(TestPlugin);
    registry.register(AlternatePlugin);

    let removed = registry.deregister(test_mode()).expect("plugin removed");
    assert_eq!(removed.descriptor().id, TEST_DESCRIPTOR.id);
    assert!(!registry.contains_mode(test_mode()));
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry.iter().next().unwrap().descriptor().id,
        ALT_DESCRIPTOR.id
    );
    assert!(registry.mode_by_id(TEST_DESCRIPTOR.id).is_none());
    assert!(registry.plugin_by_id(TEST_DESCRIPTOR.id).is_none());
}

#[test]
fn deregister_by_id_removes_plugin() {
    let mut registry = SearchPluginRegistry::empty();
    registry.register(TestPlugin);

    let removed = registry
        .deregister_by_id(TEST_DESCRIPTOR.id)
        .expect("plugin removed by id");
    assert_eq!(removed.descriptor().id, TEST_DESCRIPTOR.id);
    assert!(registry.is_empty());
}

#[test]
fn plugin_by_id_returns_plugin() {
    let mut registry = SearchPluginRegistry::empty();
    registry.register(TestPlugin);

    let plugin = registry
        .plugin_by_id(TEST_DESCRIPTOR.id)
        .expect("plugin resolved by id");
    assert_eq!(plugin.descriptor().id, TEST_DESCRIPTOR.id);
}
