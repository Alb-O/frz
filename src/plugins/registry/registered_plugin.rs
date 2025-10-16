use std::sync::Arc;

use crate::plugins::descriptors::{SearchPluginDataset, SearchPluginDescriptor};
use crate::types::SearchMode;

use super::SearchPlugin;

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
