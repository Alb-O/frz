pub mod facets;
pub mod files;

use super::SearchPluginRegistry;
use crate::plugins::descriptors::SearchPluginDescriptor;

pub(crate) fn register_builtin_plugins(registry: &mut SearchPluginRegistry) {
    registry.register(facets::FacetSearchPlugin);
    registry.register(files::FileSearchPlugin);
}

pub(crate) fn descriptors() -> &'static [&'static SearchPluginDescriptor] {
    &BUILTIN_DESCRIPTORS
}

static BUILTIN_DESCRIPTORS: [&'static SearchPluginDescriptor; 2] =
    [&facets::FACET_DESCRIPTOR, &files::FILE_DESCRIPTOR];
