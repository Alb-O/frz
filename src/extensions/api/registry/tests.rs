use super::*;
use crate::extensions::api::contributions::{
    Contribution, ExtensionPackage, Icon, IconProvider, IconResource, IconStore, PreviewSplit,
    PreviewSplitContext, PreviewSplitStore,
};
use crate::extensions::api::{
    context::{ExtensionQueryContext, ExtensionSelectionContext},
    descriptors::{
        ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
    },
    error::ExtensionCatalogError,
    search::{SearchData, SearchMode, SearchSelection, SearchStream},
};
use ratatui::{Frame, layout::Rect};
use std::sync::Arc;

struct TestDataset;

impl ExtensionDataset for TestDataset {
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

static TEST_DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
    id: "test",
    ui: ExtensionUiDefinition {
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

impl ExtensionDataset for AlternateDataset {
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

static ALT_DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
    id: "alt",
    ui: ExtensionUiDefinition {
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

struct TestModule;

impl ExtensionModule for TestModule {
    fn descriptor(&self) -> &'static ExtensionDescriptor {
        &TEST_DESCRIPTOR
    }

    fn stream(
        &self,
        _query: &str,
        _stream: SearchStream<'_>,
        _context: ExtensionQueryContext<'_>,
    ) -> bool {
        false
    }

    fn selection(
        &self,
        _context: ExtensionSelectionContext<'_>,
        _index: usize,
    ) -> Option<SearchSelection> {
        None
    }
}

struct AlternateModule;

impl ExtensionModule for AlternateModule {
    fn descriptor(&self) -> &'static ExtensionDescriptor {
        &ALT_DESCRIPTOR
    }

    fn stream(
        &self,
        _query: &str,
        _stream: SearchStream<'_>,
        _context: ExtensionQueryContext<'_>,
    ) -> bool {
        false
    }

    fn selection(
        &self,
        _context: ExtensionSelectionContext<'_>,
        _index: usize,
    ) -> Option<SearchSelection> {
        None
    }
}

struct TestPreview;

impl PreviewSplit for TestPreview {
    fn render_preview(&self, _frame: &mut Frame, _area: Rect, _context: PreviewSplitContext<'_>) {}
}

#[derive(Clone, Copy)]
struct TestIcons;

impl IconProvider for TestIcons {
    fn icon_for(&self, resource: IconResource<'_>) -> Option<Icon> {
        match resource {
            IconResource::File(_) => Some(Icon::new('x', None)),
        }
    }
}

#[derive(Clone)]
struct TestBundle {
    contributions: Vec<Contribution>,
}

impl TestBundle {
    fn search_with_preview() -> Self {
        Self {
            contributions: vec![
                Contribution::search_tab(&TEST_DESCRIPTOR, TestModule),
                Contribution::preview_split(&TEST_DESCRIPTOR, TestPreview),
                Contribution::icons(&TEST_DESCRIPTOR, TestIcons),
            ],
        }
    }

    fn preview_only() -> Self {
        Self {
            contributions: vec![Contribution::preview_split(&TEST_DESCRIPTOR, TestPreview)],
        }
    }

    fn icons_only() -> Self {
        Self {
            contributions: vec![Contribution::icons(&TEST_DESCRIPTOR, TestIcons)],
        }
    }
}

impl ExtensionPackage for TestBundle {
    type Contributions<'a>
        = std::vec::IntoIter<Contribution>
    where
        Self: 'a;

    fn contributions(&self) -> Self::Contributions<'_> {
        self.contributions.clone().into_iter()
    }
}

fn preview_split_for(
    catalog: &ExtensionCatalog,
    mode: SearchMode,
) -> Option<Arc<dyn PreviewSplit>> {
    catalog.contributions().resolve::<PreviewSplitStore>(mode)
}

fn icon_provider_for(
    catalog: &ExtensionCatalog,
    mode: SearchMode,
) -> Option<Arc<dyn IconProvider>> {
    catalog.contributions().resolve::<IconStore>(mode)
}

#[test]
fn registers_modules_in_insertion_order() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_module(TestModule)
        .expect("register test module");
    registry
        .register_module(AlternateModule)
        .expect("register alternate module");
    let modes: Vec<SearchMode> = registry.modules().map(|module| module.mode()).collect();
    assert_eq!(modes, vec![test_mode(), alt_mode()]);
}

#[test]
fn deregister_removes_module_and_updates_indexes() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_package(TestBundle::search_with_preview())
        .expect("register package with preview");
    registry
        .register_module(AlternateModule)
        .expect("register alternate module");

    let removed = registry.remove(test_mode()).expect("module removed");
    assert_eq!(removed.descriptor().id, TEST_DESCRIPTOR.id);
    assert!(!registry.contains_mode(test_mode()));
    assert_eq!(registry.len(), 1);
    assert_eq!(
        registry.modules().next().unwrap().descriptor().id,
        ALT_DESCRIPTOR.id
    );
    assert!(registry.mode_by_id(TEST_DESCRIPTOR.id).is_none());
    assert!(registry.module_by_id(TEST_DESCRIPTOR.id).is_none());
    assert!(preview_split_for(&registry, test_mode()).is_none());
    assert!(icon_provider_for(&registry, test_mode()).is_none());
}

#[test]
fn deregister_by_id_removes_module() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_package(TestBundle::search_with_preview())
        .expect("register package with preview");

    let removed = registry
        .remove_by_id(TEST_DESCRIPTOR.id)
        .expect("module removed by id");
    assert_eq!(removed.descriptor().id, TEST_DESCRIPTOR.id);
    assert!(registry.is_empty());
    assert!(preview_split_for(&registry, test_mode()).is_none());
    assert!(icon_provider_for(&registry, test_mode()).is_none());
}

#[test]
fn module_by_id_returns_module() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_module(TestModule)
        .expect("register test module");

    let module = registry
        .module_by_id(TEST_DESCRIPTOR.id)
        .expect("module resolved by id");
    assert_eq!(module.descriptor().id, TEST_DESCRIPTOR.id);
}

#[test]
fn duplicate_registration_returns_error() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_module(TestModule)
        .expect("register test module");

    let error = registry
        .register_module(TestModule)
        .expect_err("expected duplicate registration to fail");
    assert!(matches!(
        error,
        ExtensionCatalogError::DuplicateMode { .. } | ExtensionCatalogError::DuplicateId { .. }
    ));
}

#[test]
fn register_package_registers_preview_split() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_package(TestBundle::search_with_preview())
        .expect("register package");

    assert!(preview_split_for(&registry, test_mode()).is_some());
}

#[test]
fn register_package_registers_icons() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_package(TestBundle::search_with_preview())
        .expect("register package");

    assert!(icon_provider_for(&registry, test_mode()).is_some());
}

#[test]
fn duplicate_preview_split_returns_error() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_package(TestBundle::preview_only())
        .expect("register preview bundle");

    let error = registry
        .register_package(TestBundle::preview_only())
        .expect_err("expected duplicate preview split to fail");
    assert!(matches!(
        error,
        ExtensionCatalogError::ContributionConflict { contribution, .. }
            if contribution == "preview split"
    ));
}

#[test]
fn duplicate_icons_returns_error() {
    let mut registry = ExtensionCatalog::empty();
    registry
        .register_package(TestBundle::icons_only())
        .expect("register icon bundle");

    let error = registry
        .register_package(TestBundle::icons_only())
        .expect_err("expected duplicate icons to fail");
    assert!(matches!(
        error,
        ExtensionCatalogError::ContributionConflict { contribution, .. }
            if contribution == "icons"
    ));
}
