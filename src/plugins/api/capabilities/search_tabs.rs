use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::plugins::api::descriptors::SearchPluginDescriptor;
use crate::plugins::api::error::PluginRegistryError;
use crate::plugins::api::registry::{RegisteredPlugin, SearchPlugin};
use crate::plugins::api::search::SearchMode;

use super::{CapabilityInstallContext, CapabilitySpecImpl};

/// Storage backing registered search tabs.
#[derive(Clone, Default)]
pub struct SearchTabStore {
    plugins: IndexMap<SearchMode, RegisteredPlugin>,
    id_index: HashMap<&'static str, SearchMode>,
}

impl SearchTabStore {
    pub fn ensure_available(
        &self,
        descriptor: &'static SearchPluginDescriptor,
    ) -> Result<(), PluginRegistryError> {
        let mode = SearchMode::from_descriptor(descriptor);
        if self.plugins.contains_key(&mode) {
            return Err(PluginRegistryError::DuplicateMode { mode });
        }
        if self.id_index.contains_key(descriptor.id) {
            return Err(PluginRegistryError::DuplicateId { id: descriptor.id });
        }
        Ok(())
    }

    pub fn insert(&mut self, plugin: RegisteredPlugin) {
        let mode = plugin.mode();
        let descriptor = plugin.descriptor();
        let existing = self.plugins.insert(mode, plugin);
        debug_assert!(existing.is_none(), "plugins should be unique per mode");
        let previous = self.id_index.insert(descriptor.id, mode);
        debug_assert!(previous.is_none(), "plugin identifiers should be unique");
    }

    pub fn plugin(&self, mode: SearchMode) -> Option<Arc<dyn SearchPlugin>> {
        self.plugins.get(&mode).map(|plugin| plugin.plugin())
    }

    pub fn iter(&self) -> impl Iterator<Item = &RegisteredPlugin> {
        self.plugins.values()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &'static SearchPluginDescriptor> + '_ {
        self.plugins.values().map(|plugin| plugin.descriptor())
    }

    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.id_index.get(id).copied()
    }

    pub fn plugin_by_id(&self, id: &str) -> Option<Arc<dyn SearchPlugin>> {
        self.mode_by_id(id).and_then(|mode| self.plugin(mode))
    }

    pub fn remove(&mut self, mode: SearchMode) -> Option<RegisteredPlugin> {
        let removed = self.plugins.shift_remove(&mode);
        if let Some(ref plugin) = removed {
            self.id_index.remove(plugin.descriptor().id);
        }
        removed
    }

    pub fn remove_by_id(&mut self, id: &str) -> Option<(SearchMode, RegisteredPlugin)> {
        let mode = self.id_index.remove(id)?;
        let plugin = self.plugins.shift_remove(&mode)?;
        Some((mode, plugin))
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn contains_mode(&self, mode: SearchMode) -> bool {
        self.plugins.contains_key(&mode)
    }
}

/// Capability describing a search tab implementation.
#[derive(Clone)]
pub struct SearchTabCapability {
    descriptor: &'static SearchPluginDescriptor,
    plugin: Arc<dyn SearchPlugin>,
}

impl SearchTabCapability {
    pub fn new<P>(descriptor: &'static SearchPluginDescriptor, plugin: P) -> Self
    where
        P: SearchPlugin + 'static,
    {
        let plugin: Arc<dyn SearchPlugin> = Arc::new(plugin);
        Self { descriptor, plugin }
    }
}

impl CapabilitySpecImpl for SearchTabCapability {
    fn install(
        &self,
        context: &mut CapabilityInstallContext<'_>,
    ) -> Result<(), PluginRegistryError> {
        let registered = RegisteredPlugin::new(self.descriptor, Arc::clone(&self.plugin));
        context.register_search_tab(registered)
    }
}
