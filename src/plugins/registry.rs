use crate::plugins::{
    PluginQueryContext,
    PluginSelectionContext,
    descriptors::{SearchPluginDescriptor, SearchPluginDataset},
    systems::search::SearchStream,
};
use crate::types::{SearchMode, SearchSelection};
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
    plugins: Vec<RegisteredPlugin>,
    index: HashMap<SearchMode, usize>,
    id_index: HashMap<&'static str, usize>,
}

impl SearchPluginRegistry {
    /// Create an empty registry without any plugins registered.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            plugins: Vec::new(),
            index: HashMap::new(),
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
        let position = if let Some(position) = self.index.get(&mode).copied() {
            if let Some(existing) = self.plugins.get(position) {
                self.id_index.remove(existing.descriptor.id);
            }
            self.plugins[position] = entry;
            position
        } else {
            let position = self.plugins.len();
            self.index.insert(mode, position);
            self.plugins.push(entry);
            position
        };
        self.id_index.insert(descriptor.id, position);
    }

    /// Lookup a plugin servicing the requested mode.
    pub fn plugin(&self, mode: SearchMode) -> Option<Arc<dyn SearchPlugin>> {
        self.index
            .get(&mode)
            .and_then(|position| self.plugins.get(*position))
            .map(|entry| entry.plugin())
    }

    /// Iterate over all registered plugins.
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredPlugin> {
        self.plugins.iter()
    }

    /// Iterate over registered plugin descriptors.
    pub fn descriptors(&self) -> impl Iterator<Item = &'static SearchPluginDescriptor> + '_ {
        self.plugins.iter().map(|entry| entry.descriptor)
    }

    /// Attempt to resolve a mode identifier to a registered plugin.
    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.id_index
            .get(id)
            .and_then(|position| self.plugins.get(*position))
            .map(|entry| entry.mode())
    }
}

impl Default for SearchPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
