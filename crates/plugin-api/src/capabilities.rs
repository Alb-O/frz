use std::sync::Arc;

use crate::{
    descriptors::SearchPluginDescriptor,
    registry::{RegisteredPlugin, SearchPlugin},
};

/// Capabilities contributed by a plugin bundle.
#[derive(Clone)]
pub enum Capability {
    /// Contribute a search tab to the application.
    SearchTab(RegisteredPlugin),
}

impl Capability {
    /// Convenience constructor for creating a search tab capability.
    pub fn search_tab<P>(descriptor: &'static SearchPluginDescriptor, plugin: P) -> Self
    where
        P: SearchPlugin + 'static,
    {
        let plugin: Arc<dyn SearchPlugin> = Arc::new(plugin);
        Self::SearchTab(RegisteredPlugin::new(descriptor, plugin))
    }

    /// The descriptor associated with the capability.
    #[must_use]
    pub fn descriptor(&self) -> &'static SearchPluginDescriptor {
        match self {
            Self::SearchTab(plugin) => plugin.descriptor(),
        }
    }

    pub(crate) fn into_registered_plugin(self) -> RegisteredPlugin {
        match self {
            Self::SearchTab(plugin) => plugin,
        }
    }
}

/// A collection of capabilities provided by a plugin crate.
pub trait PluginBundle: Send + Sync {
    /// Iterator type yielded by [`capabilities`](Self::capabilities).
    type Capabilities<'a>: IntoIterator<Item = Capability>
    where
        Self: 'a;

    /// Enumerate the capabilities exposed by the bundle.
    fn capabilities(&self) -> Self::Capabilities<'_>;
}
