use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::extensions::api::descriptors::ExtensionDescriptor;
use crate::extensions::api::error::ExtensionCatalogError;
use crate::extensions::api::registry::{ExtensionModule, RegisteredModule};
use crate::extensions::api::search::SearchMode;

use super::{ContributionInstallContext, ContributionSpecImpl};

/// Storage backing registered search tabs.
#[derive(Clone, Default)]
pub struct SearchTabStore {
    modules: IndexMap<SearchMode, RegisteredModule>,
    id_index: HashMap<&'static str, SearchMode>,
}

impl SearchTabStore {
    pub fn ensure_available(
        &self,
        descriptor: &'static ExtensionDescriptor,
    ) -> Result<(), ExtensionCatalogError> {
        let mode = SearchMode::from_descriptor(descriptor);
        if self.modules.contains_key(&mode) {
            return Err(ExtensionCatalogError::DuplicateMode { mode });
        }
        if self.id_index.contains_key(descriptor.id) {
            return Err(ExtensionCatalogError::DuplicateId { id: descriptor.id });
        }
        Ok(())
    }

    pub fn insert(&mut self, module: RegisteredModule) {
        let mode = module.mode();
        let descriptor = module.descriptor();
        let existing = self.modules.insert(mode, module);
        debug_assert!(existing.is_none(), "modules should be unique per mode");
        let previous = self.id_index.insert(descriptor.id, mode);
        debug_assert!(previous.is_none(), "module identifiers should be unique");
    }

    pub fn module(&self, mode: SearchMode) -> Option<Arc<dyn ExtensionModule>> {
        self.modules.get(&mode).map(|module| module.module())
    }

    pub fn iter(&self) -> impl Iterator<Item = &RegisteredModule> {
        self.modules.values()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &'static ExtensionDescriptor> + '_ {
        self.modules.values().map(|module| module.descriptor())
    }

    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.id_index.get(id).copied()
    }

    pub fn module_by_id(&self, id: &str) -> Option<Arc<dyn ExtensionModule>> {
        self.mode_by_id(id).and_then(|mode| self.module(mode))
    }

    pub fn remove(&mut self, mode: SearchMode) -> Option<RegisteredModule> {
        let removed = self.modules.shift_remove(&mode);
        if let Some(ref module) = removed {
            self.id_index.remove(module.descriptor().id);
        }
        removed
    }

    pub fn remove_by_id(&mut self, id: &str) -> Option<(SearchMode, RegisteredModule)> {
        let mode = self.id_index.remove(id)?;
        let module = self.modules.shift_remove(&mode)?;
        Some((mode, module))
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn contains_mode(&self, mode: SearchMode) -> bool {
        self.modules.contains_key(&mode)
    }
}

/// Contribution describing a search tab implementation.
#[derive(Clone)]
pub struct SearchTabContribution {
    descriptor: &'static ExtensionDescriptor,
    module: Arc<dyn ExtensionModule>,
}

impl SearchTabContribution {
    pub fn new<P>(descriptor: &'static ExtensionDescriptor, module: P) -> Self
    where
        P: ExtensionModule + 'static,
    {
        let module: Arc<dyn ExtensionModule> = Arc::new(module);
        Self { descriptor, module }
    }
}

impl ContributionSpecImpl for SearchTabContribution {
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError> {
        let registered = RegisteredModule::new(self.descriptor, Arc::clone(&self.module));
        context.register_search_tab(registered)
    }
}
