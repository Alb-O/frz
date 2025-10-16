mod context;
pub mod builtin;
mod registry;

pub use registry::{SearchPlugin, SearchPluginRegistry};
pub use context::{PluginQueryContext, PluginSelectionContext};

/// Re-exported systems that plugins can leverage.
pub mod systems {
    #[cfg(feature = "fs")]
    pub use crate::systems::filesystem::plugin as filesystem;
    pub use crate::systems::search::plugin as search;
}
