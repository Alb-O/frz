use crate::plugins::api::{
    context::{PluginQueryContext, PluginSelectionContext},
    descriptors::SearchPluginDescriptor,
    search::{SearchMode, SearchSelection, SearchStream},
};

/// A pluggable search component that can provide results for a tab.
///
/// Search-specific helpers live under [`crate::plugins::api::search`], which exposes
/// functionality such as [`SearchStream`](crate::plugins::api::SearchStream) and the built-in
/// streaming helpers for common data sets. You can also opt into the filesystem
/// indexer to populate [`SearchData`](crate::plugins::api::SearchData) instances.
pub trait SearchPlugin: Send + Sync {
    /// Static descriptor advertising plugin metadata.
    fn descriptor(&self) -> &'static SearchPluginDescriptor;

    /// Identifier describing which tab this plugin services.
    fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor())
    }

    /// Execute a query against the shared [`SearchData`](crate::plugins::api::SearchData) and
    /// stream results.
    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool;

    /// Convert a filtered index into a [`SearchSelection`](crate::plugins::api::SearchSelection) for the caller.
    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection>;
}
