use std::collections::HashMap;

use frz_plugin_api::{SearchMode, descriptors::SearchPluginDescriptor};

/// Labels and titles for one of the search panes.
#[derive(Debug, Clone)]
pub struct PaneUiConfig {
    pub mode_title: String,
    pub hint: String,
    pub table_title: String,
    pub count_label: String,
}

impl PaneUiConfig {
    /// Construct a new [`PaneUiConfig`] with custom labels.
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
    /// Build a [`TabUiConfig`] from constituent pieces.
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
    index: HashMap<SearchMode, usize>,
}

impl Default for UiConfig {
    fn default() -> Self {
        let mut config = Self {
            filter_label: "Filter facets".to_string(),
            detail_panel_title: "Selection details".to_string(),
            tabs: Vec::new(),
            index: HashMap::new(),
        };
        for descriptor in crate::plugins::builtin::descriptors() {
            config.register_plugin(descriptor);
        }
        config
    }
}

impl UiConfig {
    /// UI configuration tailored to searching tags and files.
    #[must_use]
    pub fn tags_and_files() -> Self {
        let mut config = Self {
            filter_label: "Filter tag".to_string(),
            detail_panel_title: "Selection details".to_string(),
            tabs: Vec::new(),
            index: HashMap::new(),
        };

        let facets = crate::plugins::builtin::facets::mode();
        config.register_tab(TabUiConfig::new(
            facets,
            "Tags",
            PaneUiConfig::new(
                "Tag search",
                "Type to filter tags. Press Tab to view files.",
                "Matching tags",
                "Tags",
            ),
        ));

        let files = crate::plugins::builtin::files::mode();
        config.register_tab(TabUiConfig::new(
            files,
            "Files",
            PaneUiConfig::new(
                "File search",
                "Type to filter files by path or tag. Press Tab to view tags.",
                "Matching files",
                "Files",
            ),
        ));

        config
    }

    /// Register a new tab definition with this configuration.
    pub fn register_tab(&mut self, tab: TabUiConfig) {
        let mode = tab.mode;
        if let Some(position) = self.index.get(&mode).copied() {
            self.tabs[position] = tab;
        } else {
            let idx = self.tabs.len();
            self.index.insert(mode, idx);
            self.tabs.push(tab);
        }
    }

    /// Register the default UI provided by a plugin descriptor.
    pub fn register_plugin(&mut self, descriptor: &'static SearchPluginDescriptor) {
        let mode = SearchMode::from_descriptor(descriptor);
        let pane = PaneUiConfig::new(
            descriptor.ui.mode_title,
            descriptor.ui.hint,
            descriptor.ui.table_title,
            descriptor.ui.count_label,
        );
        self.register_tab(TabUiConfig::new(mode, descriptor.ui.tab_label, pane));
    }

    /// Return all registered tabs in the order they were added.
    #[must_use]
    pub fn tabs(&self) -> &[TabUiConfig] {
        &self.tabs
    }

    /// Lookup tab metadata for the provided mode.
    #[must_use]
    pub fn tab(&self, mode: SearchMode) -> Option<&TabUiConfig> {
        self.index
            .get(&mode)
            .and_then(|position| self.tabs.get(*position))
    }

    /// Lookup pane metadata for the provided mode.
    #[must_use]
    pub fn pane(&self, mode: SearchMode) -> Option<&PaneUiConfig> {
        self.tab(mode).map(|tab| &tab.pane)
    }

    /// Mutably lookup pane metadata for the provided mode.
    pub fn pane_mut(&mut self, mode: SearchMode) -> Option<&mut PaneUiConfig> {
        let position = self.index.get(&mode).copied()?;
        self.tabs.get_mut(position).map(|tab| &mut tab.pane)
    }

    /// Retrieve the label displayed on the tab itself.
    #[must_use]
    pub fn tab_label(&self, mode: SearchMode) -> Option<&str> {
        self.tab(mode).map(|tab| tab.tab_label.as_str())
    }

    /// Resolve a registered mode identifier to its [`SearchMode`].
    #[must_use]
    pub fn mode_by_id(&self, id: &str) -> Option<SearchMode> {
        self.tabs
            .iter()
            .find(|tab| tab.mode.id() == id)
            .map(|tab| tab.mode)
    }

    /// Return the pane title associated with the provided mode.
    #[must_use]
    pub fn mode_title(&self, mode: SearchMode) -> &str {
        self.pane(mode)
            .map(|pane| pane.mode_title.as_str())
            .unwrap_or("")
    }

    /// Return the hint text associated with the provided mode.
    #[must_use]
    pub fn mode_hint(&self, mode: SearchMode) -> &str {
        self.pane(mode).map(|pane| pane.hint.as_str()).unwrap_or("")
    }

    /// Return the table title associated with the provided mode.
    #[must_use]
    pub fn mode_table_title(&self, mode: SearchMode) -> &str {
        self.pane(mode)
            .map(|pane| pane.table_title.as_str())
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::builtin::{facets, files};

    #[test]
    fn tags_and_files_registers_tabs() {
        let config = UiConfig::tags_and_files();
        assert!(config.tab(facets::mode()).is_some());
        assert!(config.tab(files::mode()).is_some());
    }
}
