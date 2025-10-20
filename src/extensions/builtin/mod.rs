pub mod attributes;
pub mod files;
pub mod logger;

use crate::extensions::api::{ExtensionCatalog, ExtensionCatalogError, ExtensionDescriptor};

pub fn register_builtin_extensions(
    catalog: &mut ExtensionCatalog,
) -> Result<(), ExtensionCatalogError> {
    catalog.register_package(attributes::bundle())?;
    catalog.register_package(files::bundle())?;
    catalog.register_package(logger::bundle())?;
    Ok(())
}

pub fn descriptors() -> &'static [&'static ExtensionDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&ExtensionDescriptor; 3] = [
    &attributes::ATTRIBUTE_DESCRIPTOR,
    &files::FILE_DESCRIPTOR,
    &logger::LOGGER_DESCRIPTOR,
];
