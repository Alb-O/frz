pub mod builtin;
mod context;
mod registry;

pub use context::{PluginQueryContext, PluginSelectionContext};
pub use registry::{SearchPlugin, SearchPluginRegistry};

/// Re-exported systems that plugins can leverage.
pub mod systems {
    #[cfg(feature = "fs")]
    pub use crate::systems::filesystem::plugin as filesystem;
    pub use crate::systems::search::plugin as search;
}
