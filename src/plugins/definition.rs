use std::hash::{Hash, Hasher};

use ratatui::{Frame, layout::Rect};

use crate::types::{SearchData, UiConfig};
use crate::ui::App;

/// UI strings exposed by a search plugin.
#[derive(Debug)]
pub struct SearchPluginUi {
    pub tab_label: &'static str,
    pub mode_title: &'static str,
    pub hint: &'static str,
    pub table_title: &'static str,
    pub count_label: &'static str,
}

impl SearchPluginUi {
    pub const fn new(
        tab_label: &'static str,
        mode_title: &'static str,
        hint: &'static str,
        table_title: &'static str,
        count_label: &'static str,
    ) -> Self {
        Self {
            tab_label,
            mode_title,
            hint,
            table_title,
            count_label,
        }
    }
}

/// Behavioral hooks exported by a search plugin for the UI layer.
#[derive(Clone, Copy, Debug)]
pub struct SearchPluginBehavior {
    pub dataset_len: fn(&SearchData) -> usize,
    pub render: fn(&mut App<'_>, &mut Frame<'_>, Rect),
}

impl SearchPluginBehavior {
    pub const fn new(
        dataset_len: fn(&SearchData) -> usize,
        render: fn(&mut App<'_>, &mut Frame<'_>, Rect),
    ) -> Self {
        Self {
            dataset_len,
            render,
        }
    }
}

/// Fully describes a plugin's contribution to the UI and runtime behavior.
#[derive(Debug)]
pub struct SearchPluginDefinition {
    pub id: &'static str,
    pub ui: SearchPluginUi,
    pub behavior: SearchPluginBehavior,
}

impl SearchPluginDefinition {
    pub const fn new(
        id: &'static str,
        ui: SearchPluginUi,
        behavior: SearchPluginBehavior,
    ) -> Self {
        Self { id, ui, behavior }
    }

    pub const fn mode(&'static self) -> SearchMode {
        SearchMode::new(self)
    }
}

/// Identifier describing a plugin-provided search mode.
#[derive(Clone, Copy)]
pub struct SearchMode {
    id: &'static str,
    definition: &'static SearchPluginDefinition,
}

impl std::fmt::Debug for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SearchMode").field(&self.id).finish()
    }
}

impl SearchMode {
    pub const fn new(definition: &'static SearchPluginDefinition) -> Self {
        Self {
            id: definition.id,
            definition,
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.id
    }

    pub const fn definition(self) -> &'static SearchPluginDefinition {
        self.definition
    }

    pub fn ui(self) -> &'static SearchPluginUi {
        &self.definition.ui
    }

    pub fn behavior(self) -> &'static SearchPluginBehavior {
        &self.definition.behavior
    }
}

impl PartialOrd for SearchMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(other.as_str()))
    }
}

impl Ord for SearchMode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(other.as_str())
    }
}

impl PartialEq for SearchMode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.definition, other.definition)
    }
}

impl Eq for SearchMode {}

impl Hash for SearchMode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.definition as *const SearchPluginDefinition).hash(state);
    }
}

impl SearchMode {
    pub fn title(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.mode_title.as_str())
            .unwrap_or(self.ui().mode_title)
    }

    pub fn hint(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.hint.as_str())
            .unwrap_or(self.ui().hint)
    }

    pub fn table_title(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.table_title.as_str())
            .unwrap_or(self.ui().table_title)
    }

    pub fn count_label(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.count_label.as_str())
            .unwrap_or(self.ui().count_label)
    }
}
