use super::*;
use crate::plugins::api::capabilities::{
    Capability, PluginBundle, PreviewSplit, PreviewSplitContext,
};
use crate::plugins::api::{
    context::{PluginQueryContext, PluginSelectionContext},
    descriptors::{
        FrzPluginDataset, FrzPluginDescriptor, FrzPluginUiDefinition, TableContext, TableDescriptor,
    },
    error::PluginRegistryError,
    search::{SearchData, SearchMode, SearchSelection, SearchStream},
};
use ratatui::{Frame, layout::Rect};

struct TestDataset;

impl FrzPluginDataset for TestDataset {
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

static TEST_DESCRIPTOR: FrzPluginDescriptor = FrzPluginDescriptor {
    id: "test",
    ui: FrzPluginUiDefinition {
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

impl FrzPluginDataset for AlternateDataset {
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

static ALT_DESCRIPTOR: FrzPluginDescriptor = FrzPluginDescriptor {
    id: "alt",
    ui: FrzPluginUiDefinition {
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

impl FrzPlugin for TestPlugin {
    fn descriptor(&self) -> &'static FrzPluginDescriptor {
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

impl FrzPlugin for AlternatePlugin {
    fn descriptor(&self) -> &'static FrzPluginDescriptor {
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

struct TestPreview;

impl PreviewSplit for TestPreview {
    fn render_preview(&self, _frame: &mut Frame, _area: Rect, _context: PreviewSplitContext<'_>) {}
}

#[derive(Clone)]
struct TestBundle {
    capabilities: Vec<Capability>,
}

impl TestBundle {
    fn search_with_preview() -> Self {
        Self {
            capabilities: vec![
                Capability::search_tab(&TEST_DESCRIPTOR, TestPlugin),
                Capability::preview_split(&TEST_DESCRIPTOR, TestPreview),
            ],
        }
    }

    fn preview_only() -> Self {
        Self {
            capabilities: vec![Capability::preview_split(&TEST_DESCRIPTOR, TestPreview)],
        }
    }
}

impl PluginBundle for TestBundle {
    type Capabilities<'a>
        = std::vec::IntoIter<Capability>
    where
        Self: 'a;

    fn capabilities(&self) -> Self::Capabilities<'_> {
        self.capabilities.clone().into_iter()
    }
}

#[test]
fn registers_plugins_in_insertion_order() {
    let mut registry = FrzPluginRegistry::empty();
    registry.register(TestPlugin).expect("register test plugin");
    registry
        .register(AlternatePlugin)
        .expect("register alternate plugin");
    let modes: Vec<SearchMode> = registry.iter().map(|plugin| plugin.mode()).collect();
    assert_eq!(modes, vec![test_mode(), alt_mode()]);
}

#[test]
fn deregister_removes_plugin_and_updates_indexes() {
    let mut registry = FrzPluginRegistry::empty();
    registry
        .register_bundle(TestBundle::search_with_preview())
        .expect("register bundle with preview");
    registry
        .register(AlternatePlugin)
        .expect("register alternate plugin");

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
    assert!(registry.preview_split(test_mode()).is_none());
}

#[test]
fn deregister_by_id_removes_plugin() {
    let mut registry = FrzPluginRegistry::empty();
    registry
        .register_bundle(TestBundle::search_with_preview())
        .expect("register bundle with preview");

    let removed = registry
        .deregister_by_id(TEST_DESCRIPTOR.id)
        .expect("plugin removed by id");
    assert_eq!(removed.descriptor().id, TEST_DESCRIPTOR.id);
    assert!(registry.is_empty());
    assert!(registry.preview_split(test_mode()).is_none());
}

#[test]
fn plugin_by_id_returns_plugin() {
    let mut registry = FrzPluginRegistry::empty();
    registry.register(TestPlugin).expect("register test plugin");

    let plugin = registry
        .plugin_by_id(TEST_DESCRIPTOR.id)
        .expect("plugin resolved by id");
    assert_eq!(plugin.descriptor().id, TEST_DESCRIPTOR.id);
}

#[test]
fn duplicate_registration_returns_error() {
    let mut registry = FrzPluginRegistry::empty();
    registry.register(TestPlugin).expect("register test plugin");

    let error = registry
        .register(TestPlugin)
        .expect_err("expected duplicate registration to fail");
    assert!(matches!(
        error,
        PluginRegistryError::DuplicateMode { .. } | PluginRegistryError::DuplicateId { .. }
    ));
}

#[test]
fn register_bundle_registers_preview_split() {
    let mut registry = FrzPluginRegistry::empty();
    registry
        .register_bundle(TestBundle::search_with_preview())
        .expect("register bundle");

    assert!(registry.preview_split(test_mode()).is_some());
}

#[test]
fn duplicate_preview_split_returns_error() {
    let mut registry = FrzPluginRegistry::empty();
    registry
        .register_bundle(TestBundle::preview_only())
        .expect("register preview bundle");

    let error = registry
        .register_bundle(TestBundle::preview_only())
        .expect_err("expected duplicate preview split to fail");
    assert!(matches!(
        error,
        PluginRegistryError::CapabilityConflict { capability, .. }
            if capability == "preview split"
    ));
}
