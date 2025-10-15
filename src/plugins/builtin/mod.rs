mod facets;
mod files;

use super::SearchPluginRegistry;

pub(crate) fn register_builtin_plugins(registry: &mut SearchPluginRegistry) {
    registry.register(facets::FacetSearchPlugin);
    registry.register(files::FileSearchPlugin);
}
