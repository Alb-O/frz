use crate::plugins::{
    PluginQueryContext,
    PluginSelectionContext,
    SearchPlugin,
    systems::search::{SearchStream, stream_facets},
};
use crate::types::{SearchMode, SearchSelection};

pub(crate) struct FacetSearchPlugin;

impl SearchPlugin for FacetSearchPlugin {
    fn mode(&self) -> SearchMode {
        SearchMode::FACETS
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool {
        stream_facets(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection> {
        context
            .data()
            .facets
            .get(index)
            .cloned()
            .map(SearchSelection::Facet)
    }
}
