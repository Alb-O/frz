use std::sync::Arc;

use crate::plugins::api::{
    descriptors::{FrzPluginDataset, FrzPluginDescriptor},
    search::SearchMode,
};

use super::FrzPlugin;

/// Metadata and implementation pair stored by the registry.
#[derive(Clone)]
pub struct RegisteredPlugin {
    descriptor: &'static FrzPluginDescriptor,
    plugin: Arc<dyn FrzPlugin>,
}

impl RegisteredPlugin {
    #[must_use]
    pub fn new(descriptor: &'static FrzPluginDescriptor, plugin: Arc<dyn FrzPlugin>) -> Self {
        Self { descriptor, plugin }
    }

    #[must_use]
    pub fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor)
    }

    #[must_use]
    pub fn descriptor(&self) -> &'static FrzPluginDescriptor {
        self.descriptor
    }

    #[must_use]
    pub fn dataset(&self) -> &'static dyn FrzPluginDataset {
        self.descriptor.dataset
    }

    #[must_use]
    pub fn plugin(&self) -> Arc<dyn FrzPlugin> {
        Arc::clone(&self.plugin)
    }
}
