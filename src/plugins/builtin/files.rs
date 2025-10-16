use crate::plugins::{
    PluginQueryContext,
    PluginSelectionContext,
    SearchPlugin,
    systems::search::{SearchStream, stream_files},
};
use crate::types::{SearchMode, SearchSelection};

pub(crate) struct FileSearchPlugin;

impl SearchPlugin for FileSearchPlugin {
    fn mode(&self) -> SearchMode {
        SearchMode::FILES
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool {
        stream_files(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection> {
        context
            .data()
            .files
            .get(index)
            .cloned()
            .map(SearchSelection::File)
    }
}
