use std::collections::HashMap;
use std::sync::Arc;

use ratatui::crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::extensions::api::descriptors::ExtensionDescriptor;
use crate::extensions::api::error::ExtensionCatalogError;
use crate::extensions::api::search::{SearchData, SearchMode};

use super::{
    ContributionInstallContext, ContributionSpecImpl, Icon, PreviewResource, ScopedContribution,
};

/// Layout hints describing how a preview split should be rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewLayout {
    /// Render the preview alongside the primary results table.
    Split,
    /// Render the preview using the full available width, hiding the results table.
    PreviewOnly,
}

/// Context provided to preview split renderers when drawing the preview area.
pub struct PreviewSplitContext<'a> {
    data: &'a SearchData,
    filtered: &'a [usize],
    scores: &'a [u16],
    selected: Option<usize>,
    query: &'a str,
    bat_theme: Option<&'a str>,
    selection: Option<PreviewResource<'a>>,
    git_modifications: bool,
}

impl<'a> PreviewSplitContext<'a> {
    pub fn new(
        data: &'a SearchData,
        filtered: &'a [usize],
        scores: &'a [u16],
        selected: Option<usize>,
        selection: Option<PreviewResource<'a>>,
        query: &'a str,
        bat_theme: Option<&'a str>,
        git_modifications: bool,
    ) -> Self {
        Self {
            data,
            filtered,
            scores,
            selected,
            selection,
            query,
            bat_theme,
            git_modifications,
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

    pub fn selection(&self) -> Option<&PreviewResource<'a>> {
        self.selection.as_ref()
    }

    pub fn query(&self) -> &'a str {
        self.query
    }

    pub fn bat_theme(&self) -> Option<&'a str> {
        self.bat_theme
    }

    pub fn git_modifications(&self) -> bool {
        self.git_modifications
    }
}

/// Behaviour implemented by preview split renderers.
pub trait PreviewSplit: Send + Sync {
    fn render_preview(&self, frame: &mut Frame, area: Rect, context: PreviewSplitContext<'_>);

    fn header_icon(&self) -> Option<Icon> {
        None
    }

    fn layout(&self) -> PreviewLayout {
        PreviewLayout::Split
    }

    fn handle_key(&self, _key: KeyEvent) -> bool {
        false
    }
}

/// Storage for preview split renderers registered by extensions.
#[derive(Clone, Default)]
pub struct PreviewSplitStore {
    splits: HashMap<SearchMode, Arc<dyn PreviewSplit>>,
}

impl PreviewSplitStore {
    pub fn register(
        &mut self,
        mode: SearchMode,
        preview: Arc<dyn PreviewSplit>,
    ) -> Result<(), ExtensionCatalogError> {
        if self.splits.contains_key(&mode) {
            return Err(ExtensionCatalogError::contribution_conflict(
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

impl ScopedContribution for PreviewSplitStore {
    type Output = Arc<dyn PreviewSplit>;

    fn resolve(&self, mode: SearchMode) -> Option<Self::Output> {
        self.get(mode)
    }
}

/// Contribution describing a preview split renderer.
#[derive(Clone)]
pub struct PreviewSplitContribution {
    descriptor: &'static ExtensionDescriptor,
    preview: Arc<dyn PreviewSplit>,
}

impl PreviewSplitContribution {
    pub fn new<P>(descriptor: &'static ExtensionDescriptor, preview: P) -> Self
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

impl ContributionSpecImpl for PreviewSplitContribution {
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError> {
        let mode = SearchMode::from_descriptor(self.descriptor);
        let store = context.storage_mut::<PreviewSplitStore>();
        store.register(mode, Arc::clone(&self.preview))?;
        context.register_cleanup::<PreviewSplitStore, _>(PreviewSplitStore::remove);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::extensions::api::search::{AttributeRow, FileRow};

    #[test]
    fn context_exposes_selection_resource() {
        let data = SearchData::new()
            .with_files(vec![FileRow::new("a.rs", Vec::<String>::new())])
            .with_attributes(vec![AttributeRow::new("alpha", 1)]);
        let selection = Some(PreviewResource::File(&data.files[0]));
        let context = PreviewSplitContext::new(
            &data,
            &[0],
            &[100],
            Some(0),
            selection,
            "query",
            None,
            false,
        );
        let resource = context.selection().expect("selection present");
        match resource {
            PreviewResource::File(file) => assert_eq!(file.path, "a.rs"),
            PreviewResource::Attribute(_) => panic!("unexpected attribute"),
        }
        assert_eq!(context.selected_row_index(), Some(0));
        assert!(!context.git_modifications());
    }
}
