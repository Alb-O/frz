pub mod builtin;
mod context;
pub mod descriptors;
mod registry;

pub use context::{PluginQueryContext, PluginSelectionContext};
pub use registry::{RegisteredPlugin, SearchPlugin, SearchPluginRegistry};

/// Re-exported systems that plugins can leverage.
pub mod systems {
    #[cfg(feature = "fs")]
    pub use crate::systems::filesystem::plugin as filesystem;
    pub use crate::systems::search::plugin as search;
}
