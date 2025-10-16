mod facets;
mod files;

use super::SearchPluginRegistry;

pub use facets::{FACETS_DEFINITION, MODE as FACETS_MODE};
pub use files::{FILES_DEFINITION, MODE as FILES_MODE};

pub(crate) fn register_builtin_plugins(registry: &mut SearchPluginRegistry) {
    registry.register(facets::FacetSearchPlugin);
    registry.register(files::FileSearchPlugin);
}
