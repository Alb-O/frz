use std::sync::Arc;

use crate::extensions::api::{
    descriptors::{ExtensionDataset, ExtensionDescriptor},
    search::SearchMode,
};

use super::ExtensionModule;

/// Metadata and implementation pair stored by the catalog.
#[derive(Clone)]
pub struct RegisteredModule {
    descriptor: &'static ExtensionDescriptor,
    module: Arc<dyn ExtensionModule>,
}

impl RegisteredModule {
    #[must_use]
    pub fn new(descriptor: &'static ExtensionDescriptor, module: Arc<dyn ExtensionModule>) -> Self {
        Self { descriptor, module }
    }

    #[must_use]
    pub fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor)
    }

    #[must_use]
    pub fn descriptor(&self) -> &'static ExtensionDescriptor {
        self.descriptor
    }

    #[must_use]
    pub fn dataset(&self) -> &'static dyn ExtensionDataset {
        self.descriptor.dataset
    }

    #[must_use]
    pub fn module(&self) -> Arc<dyn ExtensionModule> {
        Arc::clone(&self.module)
    }
}
