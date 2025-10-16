pub mod builtin;

pub use frz_plugin_api::context::{self, PluginQueryContext, PluginSelectionContext};
pub use frz_plugin_api::descriptors;
pub use frz_plugin_api::registry::{self, RegisteredPlugin, SearchPlugin, SearchPluginRegistry};
pub use frz_plugin_api::{SearchStream, stream_facets, stream_files};

/// Re-exported systems that plugins can leverage.
pub mod systems {
    pub use crate::systems::filesystem::plugin as filesystem;
    pub use crate::systems::search::plugin as search;
}
