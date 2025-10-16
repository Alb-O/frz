use std::sync::Arc;

use crate::{descriptors::SearchPluginDescriptor, registry::SearchPlugin};

/// Capabilities contributed by a plugin bundle.
pub enum Capability {
    /// Contribute a search tab to the application.
    SearchTab {
        descriptor: &'static SearchPluginDescriptor,
        plugin: Arc<dyn SearchPlugin>,
    },
}

/// A collection of capabilities provided by a plugin crate.
pub trait PluginBundle: Send + Sync {
    fn capabilities(&self) -> Vec<Capability>;
}
