use std::sync::atomic::AtomicU64;

use crate::plugins::{
    systems::search::{stream_files, SearchStream},
    SearchPlugin,
};
use crate::types::{SearchData, SearchMode, SearchSelection};

pub(crate) struct FileSearchPlugin;

impl SearchPlugin for FileSearchPlugin {
    fn mode(&self) -> SearchMode {
        SearchMode::FILES
    }

    fn stream(
        &self,
        data: &SearchData,
        query: &str,
        stream: SearchStream<'_>,
        latest_query_id: &AtomicU64,
    ) -> bool {
        stream_files(data, query, stream, latest_query_id)
    }

    fn selection(&self, data: &SearchData, index: usize) -> Option<SearchSelection> {
        data.files.get(index).cloned().map(SearchSelection::File)
    }
}
