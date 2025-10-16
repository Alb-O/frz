pub mod facets {
    pub use frz_plugin_facets::*;
}

pub mod files {
    pub use frz_plugin_files::*;
}

use frz_plugin_api::{SearchPluginDescriptor, SearchPluginRegistry};

pub fn register_builtin_plugins(registry: &mut SearchPluginRegistry) {
    registry.register(facets::FacetSearchPlugin);
    registry.register(files::FileSearchPlugin);
}

pub fn descriptors() -> &'static [&'static SearchPluginDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&SearchPluginDescriptor; 2] =
    [&facets::FACET_DESCRIPTOR, &files::FILE_DESCRIPTOR];
