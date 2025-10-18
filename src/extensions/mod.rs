pub mod api;
pub mod builtin;

pub use crate::extensions::api::context::{self, ExtensionQueryContext, ExtensionSelectionContext};
pub use crate::extensions::api::descriptors;
pub use crate::extensions::api::registry::{
    self, ExtensionCatalog, ExtensionModule, RegisteredModule,
};
pub use crate::extensions::api::{SearchStream, stream_attributes, stream_files};

/// Re-exported systems that extensions can leverage.
pub mod systems {
    pub use crate::systems::filesystem::extension as filesystem;
    pub use crate::systems::search::extension as search;
}
