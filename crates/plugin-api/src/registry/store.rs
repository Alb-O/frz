use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::{descriptors::SearchPluginDescriptor, types::SearchMode};

#[cfg(feature = "capabilities")]
use crate::capabilities::{Capability, PluginBundle};

use super::{RegisteredPlugin, SearchPlugin};

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

    /// Create a registry without registering any plugins.
    #[must_use]
    pub fn new() -> Self {
        Self::empty()
    }

    /// Register or replace a plugin implementation for its declared mode.
    pub fn register<P>(&mut self, plugin: P)
    where
        P: SearchPlugin + 'static,
    {
        let descriptor = plugin.descriptor();
        let plugin = Arc::new(plugin) as Arc<dyn SearchPlugin>;
        self.insert(descriptor, plugin);
    }

    fn insert(
        &mut self,
        descriptor: &'static SearchPluginDescriptor,
        plugin: Arc<dyn SearchPlugin>,
    ) {
        let mode = SearchMode::from_descriptor(descriptor);
        let entry = RegisteredPlugin::new(descriptor, plugin);
        if let Some(existing) = self.plugins.insert(mode, entry) {
            self.id_index.remove(existing.descriptor().id);
        }
        self.id_index.insert(descriptor.id, mode);
    }

    #[cfg(feature = "capabilities")]
    pub fn register_bundle<B>(&mut self, bundle: B)
    where
        B: PluginBundle,
    {
        for capability in bundle.capabilities() {
            match capability {
                Capability::SearchTab { descriptor, plugin } => {
                    self.insert(descriptor, plugin);
                }
            }
        }
    }

    /// Lookup a plugin servicing the requested mode.
    #[must_use]
    pub fn plugin(&self, mode: SearchMode) -> Option<Arc<dyn SearchPlugin>> {
        self.plugins.get(&mode).map(|entry| entry.plugin())
    }

    /// Iterate over all registered plugins.
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredPlugin> {
        self.plugins.values()
    }

    /// Iterate over registered plugin descriptors.
    pub fn descriptors(&self) -> impl Iterator<Item = &'static SearchPluginDescriptor> + '_ {
        self.plugins.values().map(|entry| entry.descriptor())
    }

    /// Attempt to resolve a mode identifier to a registered plugin.
    #[must_use]
    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.id_index.get(id).copied()
    }

    /// Attempt to resolve a mode identifier to a registered plugin implementation.
    #[must_use]
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
    #[must_use]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns `true` when no plugins have been registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Returns `true` if a plugin has been registered for the provided mode.
    #[must_use]
    pub fn contains_mode(&self, mode: SearchMode) -> bool {
        self.plugins.contains_key(&mode)
    }
}

impl Default for SearchPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
