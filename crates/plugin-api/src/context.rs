use std::sync::atomic::AtomicU64;

use crate::search::SearchData;

/// Shared inputs provided to plugins when they are asked to stream search results.
///
/// Wrapping the shared state in a context struct makes it easier to extend the
/// available data in the future without forcing every plugin implementation to
/// adjust their method signatures. This keeps the public trait surface area more
/// stable for external plugin authors.
pub struct PluginQueryContext<'a> {
    data: &'a SearchData,
    latest_query_id: &'a AtomicU64,
}

impl<'a> PluginQueryContext<'a> {
    /// Create a new query context describing the current search invocation.
    #[must_use]
    pub fn new(data: &'a SearchData, latest_query_id: &'a AtomicU64) -> Self {
        Self {
            data,
            latest_query_id,
        }
    }

    /// Access the shared [`SearchData`] for the current application state.
    #[must_use]
    pub fn data(&self) -> &'a SearchData {
        self.data
    }

    /// Access the `AtomicU64` tracking the latest processed query identifier.
    #[must_use]
    pub fn latest_query_id(&self) -> &'a AtomicU64 {
        self.latest_query_id
    }

    /// Construct a selection context sharing this query context's state.
    #[must_use]
    pub fn selection_context(&self) -> PluginSelectionContext<'a> {
        PluginSelectionContext::new(self.data)
    }
}

/// Shared inputs provided to plugins when they are asked to convert an index
/// into a [`SearchSelection`](crate::SearchSelection).
///
/// This lightweight wrapper keeps data access orthogonal to the rest of the
/// plugin registry so that additional metadata can be introduced later without
/// impacting plugin call sites.
pub struct PluginSelectionContext<'a> {
    data: &'a SearchData,
}

impl<'a> PluginSelectionContext<'a> {
    /// Create a new selection context referencing the shared [`SearchData`].
    #[must_use]
    pub fn new(data: &'a SearchData) -> Self {
        Self { data }
    }

    /// Access the shared [`SearchData`] for the current application state.
    #[must_use]
    pub fn data(&self) -> &'a SearchData {
        self.data
    }
}
