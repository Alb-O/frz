use std::sync::Arc;

use ratatui::{Frame, layout::Rect};

use crate::{
    descriptors::SearchPluginDescriptor,
    registry::{RegisteredPlugin, SearchPlugin},
    types::SearchData,
};

/// Capabilities contributed by a plugin bundle.
#[derive(Clone)]
pub enum Capability {
    /// Contribute a search tab to the application.
    SearchTab(RegisteredPlugin),
    /// Provide a split preview for an existing tab.
    PreviewSplit(PreviewSplitCapability),
}

impl Capability {
    /// Convenience constructor for creating a search tab capability.
    pub fn search_tab<P>(descriptor: &'static SearchPluginDescriptor, plugin: P) -> Self
    where
        P: SearchPlugin + 'static,
    {
        let plugin: Arc<dyn SearchPlugin> = Arc::new(plugin);
        Self::SearchTab(RegisteredPlugin::new(descriptor, plugin))
    }

    /// Convenience constructor for creating a preview split capability.
    pub fn preview_split<P>(descriptor: &'static SearchPluginDescriptor, preview: P) -> Self
    where
        P: PreviewSplit + 'static,
    {
        Self::PreviewSplit(PreviewSplitCapability::new(descriptor, preview))
    }

    /// The descriptor associated with the capability.
    #[must_use]
    pub fn descriptor(&self) -> &'static SearchPluginDescriptor {
        match self {
            Self::SearchTab(plugin) => plugin.descriptor(),
            Self::PreviewSplit(capability) => capability.descriptor(),
        }
    }
}

/// Context provided to preview split renderers when drawing the preview area.
pub struct PreviewSplitContext<'a> {
    data: &'a SearchData,
    filtered: &'a [usize],
    scores: &'a [u16],
    selected: Option<usize>,
    query: &'a str,
}

impl<'a> PreviewSplitContext<'a> {
    /// Create a new preview split context for the current UI state.
    #[must_use]
    pub fn new(
        data: &'a SearchData,
        filtered: &'a [usize],
        scores: &'a [u16],
        selected: Option<usize>,
        query: &'a str,
    ) -> Self {
        Self {
            data,
            filtered,
            scores,
            selected,
            query,
        }
    }

    /// Access the shared [`SearchData`] backing the current UI.
    #[must_use]
    pub fn data(&self) -> &'a SearchData {
        self.data
    }

    /// Access the filtered index list representing the visible table rows.
    #[must_use]
    pub fn filtered(&self) -> &'a [usize] {
        self.filtered
    }

    /// Access the scores aligned with [`filtered`](Self::filtered).
    #[must_use]
    pub fn scores(&self) -> &'a [u16] {
        self.scores
    }

    /// Return the selected index within the filtered rows, if any.
    #[must_use]
    pub fn selected_filtered_index(&self) -> Option<usize> {
        self.selected
    }

    /// Return the selected row index from the underlying dataset, if any.
    #[must_use]
    pub fn selected_row_index(&self) -> Option<usize> {
        self.selected
            .and_then(|index| self.filtered.get(index).copied())
    }

    /// Access the current query text driving the UI.
    #[must_use]
    pub fn query(&self) -> &'a str {
        self.query
    }
}

/// Behaviour implemented by preview split renderers.
pub trait PreviewSplit: Send + Sync {
    /// Render the preview portion of the split layout.
    fn render_preview(&self, frame: &mut Frame, area: Rect, context: PreviewSplitContext<'_>);
}

/// Metadata and implementation pair describing a preview split capability.
#[derive(Clone)]
pub struct PreviewSplitCapability {
    descriptor: &'static SearchPluginDescriptor,
    preview: Arc<dyn PreviewSplit>,
}

impl PreviewSplitCapability {
    fn new<P>(descriptor: &'static SearchPluginDescriptor, preview: P) -> Self
    where
        P: PreviewSplit + 'static,
    {
        let preview: Arc<dyn PreviewSplit> = Arc::new(preview);
        Self {
            descriptor,
            preview,
        }
    }

    #[must_use]
    pub fn descriptor(&self) -> &'static SearchPluginDescriptor {
        self.descriptor
    }

    #[must_use]
    pub fn preview(&self) -> Arc<dyn PreviewSplit> {
        Arc::clone(&self.preview)
    }
}

/// A collection of capabilities provided by a plugin crate.
pub trait PluginBundle: Send + Sync {
    /// Iterator type yielded by [`capabilities`](Self::capabilities).
    type Capabilities<'a>: IntoIterator<Item = Capability>
    where
        Self: 'a;

    /// Enumerate the capabilities exposed by the bundle.
    fn capabilities(&self) -> Self::Capabilities<'_>;
}
