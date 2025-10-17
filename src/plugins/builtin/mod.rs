pub mod attributes;
pub mod files;

use crate::plugins::api::{PluginRegistryError, SearchPluginDescriptor, SearchPluginRegistry};

pub fn register_builtin_plugins(
    registry: &mut SearchPluginRegistry,
) -> Result<(), PluginRegistryError> {
    registry.register_bundle(attributes::bundle())?;
    registry.register_bundle(files::bundle())?;
    Ok(())
}

pub fn descriptors() -> &'static [&'static SearchPluginDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&SearchPluginDescriptor; 2] =
    [&attributes::ATTRIBUTE_DESCRIPTOR, &files::FILE_DESCRIPTOR];
