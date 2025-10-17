use std::sync::Arc;

use crate::capabilities::{
    Capability, CapabilityInstallContext, CapabilityRegistry, PluginBundle, PreviewSplit,
    PreviewSplitStore, SearchTabStore,
};
use crate::descriptors::SearchPluginDescriptor;
use crate::error::PluginRegistryError;
use crate::registry::RegisteredPlugin;
use crate::search::SearchMode;

use super::SearchPlugin;

/// Registry of all search plugins contributing to the current UI.
#[derive(Clone)]
pub struct SearchPluginRegistry {
    search_tabs: SearchTabStore,
    capabilities: CapabilityRegistry,
}

impl SearchPluginRegistry {
    /// Create an empty registry without any plugins registered.
    pub fn empty() -> Self {
        Self {
            search_tabs: SearchTabStore::default(),
            capabilities: CapabilityRegistry::new(),
        }
    }

    /// Create a registry without registering any plugins.
    pub fn new() -> Self {
        Self::empty()
    }

    fn install_capability(&mut self, capability: &Capability) -> Result<(), PluginRegistryError> {
        let mut context =
            CapabilityInstallContext::new(&mut self.search_tabs, &mut self.capabilities);
        capability.install(&mut context)
    }

    /// Register a plugin implementation for its declared mode.
    pub fn register<P>(&mut self, plugin: P) -> Result<(), PluginRegistryError>
    where
        P: SearchPlugin + 'static,
    {
        let capability = Capability::search_tab(plugin.descriptor(), plugin);
        self.install_capability(&capability)
    }

    /// Register a capability bundle.
    pub fn register_bundle<B>(&mut self, bundle: B) -> Result<(), PluginRegistryError>
    where
        B: PluginBundle,
    {
        for capability in bundle.capabilities() {
            self.install_capability(&capability)?;
        }
        Ok(())
    }

    /// Lookup a plugin servicing the requested mode.
    pub fn plugin(&self, mode: SearchMode) -> Option<Arc<dyn SearchPlugin>> {
        self.search_tabs.plugin(mode)
    }

    /// Iterate over all registered plugins.
    pub fn iter(&self) -> impl Iterator<Item = &RegisteredPlugin> {
        self.search_tabs.iter()
    }

    /// Iterate over registered plugin descriptors.
    pub fn descriptors(&self) -> impl Iterator<Item = &'static SearchPluginDescriptor> + '_ {
        self.search_tabs.descriptors()
    }

    /// Attempt to resolve a mode identifier to a registered plugin.
    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.search_tabs.mode_by_id(id)
    }

    /// Attempt to resolve a mode identifier to a registered plugin implementation.
    pub fn plugin_by_id(&self, id: &str) -> Option<Arc<dyn SearchPlugin>> {
        self.search_tabs.plugin_by_id(id)
    }

    /// Remove the plugin registered for the provided mode.
    pub fn deregister(&mut self, mode: SearchMode) -> Option<RegisteredPlugin> {
        let removed = self.search_tabs.remove(mode);
        if removed.is_some() {
            self.capabilities.remove_mode(mode);
        }
        removed
    }

    /// Remove the plugin registered for the provided identifier.
    pub fn deregister_by_id(&mut self, id: &str) -> Option<RegisteredPlugin> {
        let (mode, plugin) = self.search_tabs.remove_by_id(id)?;
        self.capabilities.remove_mode(mode);
        Some(plugin)
    }

    /// Return the number of registered plugins.
    pub fn len(&self) -> usize {
        self.search_tabs.len()
    }

    /// Returns `true` when no plugins have been registered.
    pub fn is_empty(&self) -> bool {
        self.search_tabs.is_empty()
    }

    /// Returns `true` if a plugin has been registered for the provided mode.
    pub fn contains_mode(&self, mode: SearchMode) -> bool {
        self.search_tabs.contains_mode(mode)
    }

    /// Lookup the preview split renderer registered for the requested mode.
    pub fn preview_split(&self, mode: SearchMode) -> Option<Arc<dyn PreviewSplit>> {
        self.capabilities
            .storage::<PreviewSplitStore>()
            .and_then(|store| store.get(mode))
    }
}

impl Default for SearchPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
