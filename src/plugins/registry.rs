use crate::plugins::{
    PluginQueryContext, PluginSelectionContext,
    descriptors::{SearchPluginDataset, SearchPluginDescriptor},
    systems::search::SearchStream,
};
use crate::types::{SearchMode, SearchSelection};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

/// A pluggable search component that can provide results for a tab.
///
/// Search-specific helpers live under [`crate::plugins::systems::search`], which
/// exposes functionality such as [`SearchStream`](crate::plugins::systems::search::SearchStream)
/// and the built-in streaming helpers for common data sets. When built with the
/// `fs` feature you can also opt into the filesystem indexer via
/// [`crate::plugins::systems::filesystem`], which provides helpers for spawning
/// the index worker and merging updates into [`SearchData`].
pub trait SearchPlugin: Send + Sync {
    /// Static descriptor advertising plugin metadata.
    fn descriptor(&self) -> &'static SearchPluginDescriptor;

    /// Identifier describing which tab this plugin services.
    fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor())
    }

    /// Execute a query against the shared [`SearchData`](crate::types::SearchData) and
    /// stream results.
    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool;

    /// Convert a filtered index into a [`SearchSelection`] for the caller.
    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection>;
}

/// Metadata and implementation pair stored by the registry.
#[derive(Clone)]
pub struct RegisteredPlugin {
    descriptor: &'static SearchPluginDescriptor,
    plugin: Arc<dyn SearchPlugin>,
}

impl RegisteredPlugin {
    pub fn new(descriptor: &'static SearchPluginDescriptor, plugin: Arc<dyn SearchPlugin>) -> Self {
        Self { descriptor, plugin }
    }

    pub fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor)
    }

    pub fn descriptor(&self) -> &'static SearchPluginDescriptor {
        self.descriptor
    }

    pub fn dataset(&self) -> &'static dyn SearchPluginDataset {
        self.descriptor.dataset
    }

    pub fn plugin(&self) -> Arc<dyn SearchPlugin> {
        Arc::clone(&self.plugin)
    }
}

/// Registry of all search plugins contributing to the current UI.
#[derive(Clone)]
pub struct SearchPluginRegistry {
    plugins: IndexMap<SearchMode, RegisteredPlugin>,
    id_index: HashMap<&'static str, SearchMode>,
}

impl SearchPluginRegistry {
    /// Create an empty registry without any plugins registered.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            plugins: IndexMap::new(),
            id_index: HashMap::new(),
        }
    }

    /// Create a registry populated with the built-in plugins.
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self::empty();
        super::builtin::register_builtin_plugins(&mut registry);
        registry
    }

    /// Register or replace a plugin implementation for its declared mode.
    pub fn register<P>(&mut self, plugin: P)
    where
        P: SearchPlugin + 'static,
    {
        let descriptor = plugin.descriptor();
        let mode = SearchMode::from_descriptor(descriptor);
        let plugin = Arc::new(plugin) as Arc<dyn SearchPlugin>;
        let entry = RegisteredPlugin::new(descriptor, plugin);
        if let Some(existing) = self.plugins.insert(mode, entry) {
            self.id_index.remove(existing.descriptor.id);
        }
        self.id_index.insert(descriptor.id, mode);
    }

    /// Lookup a plugin servicing the requested mode.
    pub fn plugin(&self, mode: SearchMode) -> Option<Arc<dyn SearchPlugin>> {
        self.plugins.get(&mode).map(|entry| entry.plugin())
    }

    /// Iterate over all registered plugins.
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredPlugin> {
        self.plugins.values()
    }

    /// Iterate over registered plugin descriptors.
    pub fn descriptors(&self) -> impl Iterator<Item = &'static SearchPluginDescriptor> + '_ {
        self.plugins.values().map(|entry| entry.descriptor)
    }

    /// Attempt to resolve a mode identifier to a registered plugin.
    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.id_index.get(id).copied()
    }

    /// Attempt to resolve a mode identifier to a registered plugin implementation.
    pub fn plugin_by_id(&self, id: &str) -> Option<Arc<dyn SearchPlugin>> {
        self.mode_by_id(id).and_then(|mode| self.plugin(mode))
    }

    /// Remove the plugin registered for the provided mode.
    pub fn deregister(&mut self, mode: SearchMode) -> Option<RegisteredPlugin> {
        let removed = self.plugins.shift_remove(&mode);
        if let Some(ref plugin) = removed {
            self.id_index.remove(plugin.descriptor().id);
        }
        removed
    }

    /// Remove the plugin registered for the provided identifier.
    pub fn deregister_by_id(&mut self, id: &str) -> Option<RegisteredPlugin> {
        let mode = self.id_index.remove(id)?;
        self.plugins.shift_remove(&mode)
    }

    /// Return the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns `true` when no plugins have been registered.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Returns `true` if a plugin has been registered for the provided mode.
    pub fn contains_mode(&self, mode: SearchMode) -> bool {
        self.plugins.contains_key(&mode)
    }
}

impl Default for SearchPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::descriptors::{SearchPluginUiDefinition, TableContext, TableDescriptor};
    use crate::types::{SearchData, SearchSelection};

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
}
