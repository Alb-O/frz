pub mod attributes {
    pub use frz_plugins_attributes::*;
}

pub mod files {
    pub use frz_plugins_files::*;
}

use frz_plugin_api::{PluginRegistryError, SearchPluginDescriptor, SearchPluginRegistry};

pub fn register_builtin_plugins(
    registry: &mut SearchPluginRegistry,
) -> Result<(), PluginRegistryError> {
    registry.register(attributes::AttributeSearchPlugin)?;
    registry.register(files::FileSearchPlugin)?;
    Ok(())
}

pub fn descriptors() -> &'static [&'static SearchPluginDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&SearchPluginDescriptor; 2] =
    [&attributes::ATTRIBUTE_DESCRIPTOR, &files::FILE_DESCRIPTOR];
