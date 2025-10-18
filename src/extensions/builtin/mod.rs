pub mod attributes;
pub mod files;

use crate::extensions::api::{ExtensionCatalog, ExtensionCatalogError, ExtensionDescriptor};

pub fn register_builtin_extensions(
    catalog: &mut ExtensionCatalog,
) -> Result<(), ExtensionCatalogError> {
    catalog.register_package(attributes::bundle())?;
    catalog.register_package(files::bundle())?;
    Ok(())
}

pub fn descriptors() -> &'static [&'static ExtensionDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&ExtensionDescriptor; 2] =
    [&attributes::ATTRIBUTE_DESCRIPTOR, &files::FILE_DESCRIPTOR];
