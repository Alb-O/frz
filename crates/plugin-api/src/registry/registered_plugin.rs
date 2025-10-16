use std::sync::Arc;

use crate::{
    descriptors::{SearchPluginDataset, SearchPluginDescriptor},
    types::SearchMode,
};

use super::SearchPlugin;

/// Metadata and implementation pair stored by the registry.
#[derive(Clone)]
pub struct RegisteredPlugin {
    descriptor: &'static SearchPluginDescriptor,
    plugin: Arc<dyn SearchPlugin>,
}

impl RegisteredPlugin {
    #[must_use]
    pub fn new(descriptor: &'static SearchPluginDescriptor, plugin: Arc<dyn SearchPlugin>) -> Self {
        Self { descriptor, plugin }
    }

    #[must_use]
    pub fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor)
    }

    #[must_use]
    pub fn descriptor(&self) -> &'static SearchPluginDescriptor {
        self.descriptor
    }

    #[must_use]
    pub fn dataset(&self) -> &'static dyn SearchPluginDataset {
        self.descriptor.dataset
    }

    #[must_use]
    pub fn plugin(&self) -> Arc<dyn SearchPlugin> {
        Arc::clone(&self.plugin)
    }
}
