use std::collections::HashMap;
use std::sync::Arc;

use ratatui::{Frame, layout::Rect};

use crate::plugins::api::descriptors::SearchPluginDescriptor;
use crate::plugins::api::error::PluginRegistryError;
use crate::plugins::api::search::{SearchData, SearchMode};

use super::{CapabilityInstallContext, CapabilitySpecImpl};

/// Context provided to preview split renderers when drawing the preview area.
pub struct PreviewSplitContext<'a> {
    data: &'a SearchData,
    filtered: &'a [usize],
    scores: &'a [u16],
    selected: Option<usize>,
    query: &'a str,
}

impl<'a> PreviewSplitContext<'a> {
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

    pub fn data(&self) -> &'a SearchData {
        self.data
    }

    pub fn filtered(&self) -> &'a [usize] {
        self.filtered
    }

    pub fn scores(&self) -> &'a [u16] {
        self.scores
    }

    pub fn selected_filtered_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected_row_index(&self) -> Option<usize> {
        self.selected
            .and_then(|index| self.filtered.get(index).copied())
    }

    pub fn query(&self) -> &'a str {
        self.query
    }
}

/// Behaviour implemented by preview split renderers.
pub trait PreviewSplit: Send + Sync {
    fn render_preview(&self, frame: &mut Frame, area: Rect, context: PreviewSplitContext<'_>);
}

/// Storage for preview split renderers registered by plugins.
#[derive(Clone, Default)]
pub struct PreviewSplitStore {
    splits: HashMap<SearchMode, Arc<dyn PreviewSplit>>,
}

impl PreviewSplitStore {
    pub fn register(
        &mut self,
        mode: SearchMode,
        preview: Arc<dyn PreviewSplit>,
    ) -> Result<(), PluginRegistryError> {
        if self.splits.contains_key(&mode) {
            return Err(PluginRegistryError::capability_conflict(
                "preview split",
                mode,
            ));
        }
        self.splits.insert(mode, preview);
        Ok(())
    }

    pub fn get(&self, mode: SearchMode) -> Option<Arc<dyn PreviewSplit>> {
        self.splits.get(&mode).cloned()
    }

    pub fn remove(&mut self, mode: SearchMode) {
        self.splits.remove(&mode);
    }
}

/// Capability describing a preview split renderer.
#[derive(Clone)]
pub struct PreviewSplitCapability {
    descriptor: &'static SearchPluginDescriptor,
    preview: Arc<dyn PreviewSplit>,
}

impl PreviewSplitCapability {
    pub fn new<P>(descriptor: &'static SearchPluginDescriptor, preview: P) -> Self
    where
        P: PreviewSplit + 'static,
    {
        let preview: Arc<dyn PreviewSplit> = Arc::new(preview);
        Self {
            descriptor,
            preview,
        }
    }
}

impl CapabilitySpecImpl for PreviewSplitCapability {
    fn install(
        &self,
        context: &mut CapabilityInstallContext<'_>,
    ) -> Result<(), PluginRegistryError> {
        let mode = SearchMode::from_descriptor(self.descriptor);
        let store = context.storage_mut::<PreviewSplitStore>();
        store.register(mode, Arc::clone(&self.preview))?;
        context.register_cleanup::<PreviewSplitStore, _>(PreviewSplitStore::remove);
        Ok(())
    }
}
