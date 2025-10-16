use crate::plugins::{
    PluginQueryContext, PluginSelectionContext, descriptors::SearchPluginDescriptor,
    systems::search::SearchStream,
};
use crate::types::{SearchMode, SearchSelection};

/// A pluggable search component that can provide results for a tab.
///
/// Search-specific helpers live under [`crate::plugins::systems::search`], which
/// exposes functionality such as [`SearchStream`](crate::plugins::systems::search::SearchStream)
/// and the built-in streaming helpers for common data sets. When built with the
/// `fs` feature you can also opt into the filesystem indexer via
/// [`crate::plugins::systems::filesystem`], which provides helpers for spawning
/// the index worker and merging updates into [`SearchData`].
pub trait SearchPlugin: Send + Sync {
    /// Static descriptor advertising plugin metadata.
    fn descriptor(&self) -> &'static SearchPluginDescriptor;

    /// Identifier describing which tab this plugin services.
    fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor())
    }

    /// Execute a query against the shared [`SearchData`](crate::types::SearchData) and
    /// stream results.
    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool;

    /// Convert a filtered index into a [`SearchSelection`] for the caller.
    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection>;
}
