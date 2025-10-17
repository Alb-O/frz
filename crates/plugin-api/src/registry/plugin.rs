use crate::{
    context::{PluginQueryContext, PluginSelectionContext},
    descriptors::SearchPluginDescriptor,
    search::{SearchMode, SearchSelection, SearchStream},
};

/// A pluggable search component that can provide results for a tab.
///
/// Search-specific helpers live under [`crate::search`], which exposes
/// functionality such as [`SearchStream`](crate::SearchStream) and the built-in
/// streaming helpers for common data sets. You can also opt into the filesystem
/// indexer to populate [`SearchData`] instances.
pub trait SearchPlugin: Send + Sync {
    /// Static descriptor advertising plugin metadata.
    fn descriptor(&self) -> &'static SearchPluginDescriptor;

    /// Identifier describing which tab this plugin services.
    fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor())
    }

    /// Execute a query against the shared [`SearchData`](crate::SearchData) and
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
