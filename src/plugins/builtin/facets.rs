use std::sync::atomic::AtomicU64;

use crate::plugins::{
    systems::search::{stream_facets, SearchStream},
    SearchPlugin,
};
use crate::types::{SearchData, SearchMode, SearchSelection};

pub(crate) struct FacetSearchPlugin;

impl SearchPlugin for FacetSearchPlugin {
    fn mode(&self) -> SearchMode {
        SearchMode::FACETS
    }

    fn stream(
        &self,
        data: &SearchData,
        query: &str,
        stream: SearchStream<'_>,
        latest_query_id: &AtomicU64,
    ) -> bool {
        stream_facets(data, query, stream, latest_query_id)
    }

    fn selection(&self, data: &SearchData, index: usize) -> Option<SearchSelection> {
        data.facets.get(index).cloned().map(SearchSelection::Facet)
    }
}
