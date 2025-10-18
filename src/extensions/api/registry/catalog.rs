use std::sync::Arc;

use crate::extensions::api::contributions::{
    Contribution, ContributionInstallContext, ContributionRegistry, ExtensionPackage, PreviewSplit,
    PreviewSplitStore, SearchTabStore,
};
use crate::extensions::api::descriptors::ExtensionDescriptor;
use crate::extensions::api::error::ExtensionCatalogError;
use crate::extensions::api::registry::RegisteredModule;
use crate::extensions::api::search::SearchMode;

use super::ExtensionModule;

/// Catalog of all search extensions contributing to the current UI.
#[derive(Clone)]
pub struct ExtensionCatalog {
    search_tabs: SearchTabStore,
    contributions: ContributionRegistry,
}

impl ExtensionCatalog {
    /// Create an empty catalog without any modules registered.
    pub fn empty() -> Self {
        Self {
            search_tabs: SearchTabStore::default(),
            contributions: ContributionRegistry::new(),
        }
    }

    /// Create a catalog without registering any modules.
    pub fn new() -> Self {
        Self::empty()
    }

    fn install_contribution(
        &mut self,
        contribution: &Contribution,
    ) -> Result<(), ExtensionCatalogError> {
        let mut context =
            ContributionInstallContext::new(&mut self.search_tabs, &mut self.contributions);
        contribution.install(&mut context)
    }

    /// Register a module implementation for its declared mode.
    pub fn register_module<P>(&mut self, module: P) -> Result<(), ExtensionCatalogError>
    where
        P: ExtensionModule + 'static,
    {
        let contribution = Contribution::search_tab(module.descriptor(), module);
        self.install_contribution(&contribution)
    }

    /// Register a package of contributions.
    pub fn register_package<B>(&mut self, package: B) -> Result<(), ExtensionCatalogError>
    where
        B: ExtensionPackage,
    {
        for contribution in package.contributions() {
            self.install_contribution(&contribution)?;
        }
        Ok(())
    }

    /// Lookup the module servicing the requested mode.
    pub fn module(&self, mode: SearchMode) -> Option<Arc<dyn ExtensionModule>> {
        self.search_tabs.module(mode)
    }

    /// Iterate over all registered modules.
    pub fn modules(&self) -> impl Iterator<Item = &RegisteredModule> {
        self.search_tabs.iter()
    }

    /// Iterate over registered module descriptors.
    pub fn descriptors(&self) -> impl Iterator<Item = &'static ExtensionDescriptor> + '_ {
        self.search_tabs.descriptors()
    }

    /// Attempt to resolve a mode identifier to a registered module.
    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.search_tabs.mode_by_id(id)
    }

    /// Attempt to resolve a mode identifier to a registered module implementation.
    pub fn module_by_id(&self, id: &str) -> Option<Arc<dyn ExtensionModule>> {
        self.search_tabs.module_by_id(id)
    }

    /// Remove the module registered for the provided mode.
    pub fn remove(&mut self, mode: SearchMode) -> Option<RegisteredModule> {
        let removed = self.search_tabs.remove(mode);
        if removed.is_some() {
            self.contributions.remove_mode(mode);
        }
        removed
    }

    /// Remove the module registered for the provided identifier.
    pub fn remove_by_id(&mut self, id: &str) -> Option<RegisteredModule> {
        let (mode, module) = self.search_tabs.remove_by_id(id)?;
        self.contributions.remove_mode(mode);
        Some(module)
    }

    /// Return the number of registered modules.
    pub fn len(&self) -> usize {
        self.search_tabs.len()
    }

    /// Returns `true` when no modules have been registered.
    pub fn is_empty(&self) -> bool {
        self.search_tabs.is_empty()
    }

    /// Returns `true` if a module has been registered for the provided mode.
    pub fn contains_mode(&self, mode: SearchMode) -> bool {
        self.search_tabs.contains_mode(mode)
    }

    /// Lookup the preview split renderer registered for the requested mode.
    pub fn preview_split(&self, mode: SearchMode) -> Option<Arc<dyn PreviewSplit>> {
        self.contributions
            .storage::<PreviewSplitStore>()
            .and_then(|store| store.get(mode))
    }
}

impl Default for ExtensionCatalog {
    fn default() -> Self {
        Self::new()
    }
}
