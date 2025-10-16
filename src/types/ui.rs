use std::collections::HashMap;

use crate::plugins::{SearchMode, SearchPluginDefinition};

/// Labels and titles for one of the search panes.
#[derive(Debug, Clone)]
pub struct PaneUiConfig {
    pub mode_title: String,
    pub hint: String,
    pub table_title: String,
    pub count_label: String,
}

impl PaneUiConfig {
    #[must_use]
    pub fn new(
        mode_title: impl Into<String>,
        hint: impl Into<String>,
        table_title: impl Into<String>,
        count_label: impl Into<String>,
    ) -> Self {
        Self {
            mode_title: mode_title.into(),
            hint: hint.into(),
            table_title: table_title.into(),
            count_label: count_label.into(),
        }
    }
}

/// Full UI definition for a contributed tab.
#[derive(Debug, Clone)]
pub struct TabUiConfig {
    pub mode: SearchMode,
    pub tab_label: String,
    pub pane: PaneUiConfig,
}

impl TabUiConfig {
    #[must_use]
    pub fn new(mode: SearchMode, tab_label: impl Into<String>, pane: PaneUiConfig) -> Self {
        Self {
            mode,
            tab_label: tab_label.into(),
            pane,
        }
    }
}

/// Text used by the UI when rendering pane content and tab labels.
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub filter_label: String,
    pub detail_panel_title: String,
    tabs: Vec<TabUiConfig>,
    index: HashMap<&'static str, usize>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            filter_label: "Filter".to_string(),
            detail_panel_title: "Selection details".to_string(),
            tabs: Vec::new(),
            index: HashMap::new(),
        }
    }
}

impl UiConfig {
    #[must_use]
    pub fn for_definitions<I>(definitions: I) -> Self
    where
        I: IntoIterator<Item = &'static SearchPluginDefinition>,
    {
        let mut config = Self::default();
        for definition in definitions {
            config.register_definition(definition);
        }
        config
    }

    pub fn register_definition(&mut self, definition: &'static SearchPluginDefinition) {
        let mode = definition.mode();
        let ui = &definition.ui;
        let pane =
            PaneUiConfig::new(ui.mode_title, ui.hint, ui.table_title, ui.count_label);
        self.register_tab(TabUiConfig::new(mode, ui.tab_label, pane));
    }

    pub fn register_mode(&mut self, mode: SearchMode) {
        let ui = mode.ui();
        let pane = PaneUiConfig::new(ui.mode_title, ui.hint, ui.table_title, ui.count_label);
        self.register_tab(TabUiConfig::new(mode, ui.tab_label, pane));
    }

    pub fn register_tab(&mut self, tab: TabUiConfig) {
        let id = tab.mode.as_str();
        if let Some(position) = self.index.get(id).copied() {
            self.tabs[position] = tab;
        } else {
            let idx = self.tabs.len();
            self.index.insert(id, idx);
            self.tabs.push(tab);
        }
    }

    #[must_use]
    pub fn tabs(&self) -> &[TabUiConfig] {
        &self.tabs
    }

    #[must_use]
    pub fn tab(&self, mode: SearchMode) -> Option<&TabUiConfig> {
        self.index
            .get(mode.as_str())
            .and_then(|position| self.tabs.get(*position))
    }

    #[must_use]
    pub fn pane(&self, mode: SearchMode) -> Option<&PaneUiConfig> {
        self.tab(mode).map(|tab| &tab.pane)
    }

    pub fn pane_mut(&mut self, mode: SearchMode) -> Option<&mut PaneUiConfig> {
        let position = self.index.get(mode.as_str()).copied()?;
        self.tabs.get_mut(position).map(|tab| &mut tab.pane)
    }

    #[must_use]
    pub fn tab_label(&self, mode: SearchMode) -> Option<&str> {
        self.tab(mode).map(|tab| tab.tab_label.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{SearchPluginBehavior, SearchPluginDefinition, SearchPluginUi};
    use crate::types::SearchData;
    use crate::ui::App;
    use ratatui::layout::Rect;

    fn noop_render(_app: &mut App<'_>, _frame: &mut ratatui::Frame<'_>, _area: Rect) {}

    fn dataset_len(_data: &SearchData) -> usize {
        0
    }

    static TEST_DEFINITION: SearchPluginDefinition = SearchPluginDefinition {
        id: "test",
        ui: SearchPluginUi::new("Test", "Title", "Hint", "Table", "Count"),
        behavior: SearchPluginBehavior::new(dataset_len, noop_render),
    };

    #[test]
    fn register_definition_populates_tab() {
        let config = UiConfig::for_definitions([&TEST_DEFINITION]);
        let mode = TEST_DEFINITION.mode();
        assert_eq!(config.tab_label(mode), Some("Test"));
        assert_eq!(mode.title(&config), "Title");
    }
}
