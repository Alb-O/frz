pub mod attributes;
pub mod files;

use crate::plugins::api::{FrzPluginDescriptor, FrzPluginRegistry, PluginRegistryError};

pub fn register_builtin_plugins(
    registry: &mut FrzPluginRegistry,
) -> Result<(), PluginRegistryError> {
    registry.register_bundle(attributes::bundle())?;
    registry.register_bundle(files::bundle())?;
    Ok(())
}

pub fn descriptors() -> &'static [&'static FrzPluginDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&FrzPluginDescriptor; 2] =
    [&attributes::ATTRIBUTE_DESCRIPTOR, &files::FILE_DESCRIPTOR];
