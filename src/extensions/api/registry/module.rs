use crate::extensions::api::{
    context::{ExtensionQueryContext, ExtensionSelectionContext},
    descriptors::ExtensionDescriptor,
    search::{SearchMode, SearchSelection, SearchStream},
};

/// A pluggable search component that can provide results for a tab.
///
/// Search-specific helpers live under [`crate::extensions::api::search`], which exposes
/// functionality such as [`SearchStream`](crate::extensions::api::SearchStream) and the built-in
/// streaming helpers for common data sets. You can also opt into the filesystem
/// indexer to populate [`SearchData`](crate::extensions::api::SearchData) instances.
pub trait ExtensionModule: Send + Sync {
    /// Static descriptor advertising extension metadata.
    fn descriptor(&self) -> &'static ExtensionDescriptor;

    /// Identifier describing which tab this extension services.
    fn mode(&self) -> SearchMode {
        SearchMode::from_descriptor(self.descriptor())
    }

    /// Execute a query against the shared [`SearchData`](crate::extensions::api::SearchData) and
    /// stream results.
    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: ExtensionQueryContext<'_>,
    ) -> bool;

    /// Convert a filtered index into a [`SearchSelection`](crate::extensions::api::SearchSelection) for the caller.
    fn selection(
        &self,
        context: ExtensionSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection>;
}
