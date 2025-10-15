use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crate::systems::search::SearchStream;
use crate::types::{SearchData, SearchMode, SearchSelection};

/// A pluggable search component that can provide results for a tab.
pub trait SearchPlugin: Send + Sync {
    /// Identifier describing which tab this plugin services.
    fn mode(&self) -> SearchMode;

    /// Execute a query against the shared [`SearchData`] and stream results.
    fn stream(
        &self,
        data: &SearchData,
        query: &str,
        stream: SearchStream<'_>,
        latest_query_id: &AtomicU64,
    ) -> bool;

    /// Convert a filtered index into a [`SearchSelection`] for the caller.
    fn selection(&self, data: &SearchData, index: usize) -> Option<SearchSelection>;
}

/// Registry of all search plugins contributing to the current UI.
#[derive(Clone)]
pub struct SearchPluginRegistry {
    plugins: Vec<Arc<dyn SearchPlugin>>,
    index: HashMap<SearchMode, usize>,
}

impl SearchPluginRegistry {
    /// Create an empty registry without any plugins registered.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            plugins: Vec::new(),
            index: HashMap::new(),
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
        let mode = plugin.mode();
        let plugin = Arc::new(plugin) as Arc<dyn SearchPlugin>;
        if let Some(position) = self.index.get(&mode).copied() {
            self.plugins[position] = plugin;
        } else {
            let position = self.plugins.len();
            self.index.insert(mode, position);
            self.plugins.push(plugin);
        }
    }

    /// Lookup a plugin servicing the requested mode.
    pub fn plugin(&self, mode: SearchMode) -> Option<Arc<dyn SearchPlugin>> {
        self.index
            .get(&mode)
            .and_then(|position| self.plugins.get(*position).cloned())
    }

    /// Iterate over all registered plugins.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn SearchPlugin>> {
        self.plugins.iter()
    }
}

impl Default for SearchPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
