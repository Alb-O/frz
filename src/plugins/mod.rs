pub mod api;
pub mod builtin;

pub use crate::plugins::api::context::{self, PluginQueryContext, PluginSelectionContext};
pub use crate::plugins::api::descriptors;
pub use crate::plugins::api::registry::{self, FrzPlugin, FrzPluginRegistry, RegisteredPlugin};
pub use crate::plugins::api::{SearchStream, stream_attributes, stream_files};

/// Re-exported systems that plugins can leverage.
pub mod systems {
    pub use crate::systems::filesystem::plugin as filesystem;
    pub use crate::systems::search::plugin as search;
}
