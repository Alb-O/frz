use std::collections::HashMap;

/// Identifies a single tab contributed to the search UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SearchMode {
    id: &'static str,
}

impl SearchMode {
    /// Built-in tab that exposes available facets.
    pub const FACETS: Self = Self::new("facets");
    /// Built-in tab that displays matched files.
    pub const FILES: Self = Self::new("files");

    /// Create a new search mode identifier.
    #[must_use]
    pub const fn new(id: &'static str) -> Self {
        Self { id }
    }

    /// Return the identifier for this mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.id
    }

    /// Return the active pane title for the current mode.
    #[must_use]
    pub fn title(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.mode_title.as_str())
            .unwrap_or("")
    }

    /// Return the hint text for the current mode.
    #[must_use]
    pub fn hint(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.hint.as_str())
            .unwrap_or("")
    }

    /// Return the table title for the current mode.
    #[must_use]
    pub fn table_title(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.table_title.as_str())
            .unwrap_or("")
    }

    /// Return the count label for the current mode.
    #[must_use]
    pub fn count_label(self, ui: &UiConfig) -> &str {
        ui.pane(self)
            .map(|pane| pane.count_label.as_str())
            .unwrap_or("")
    }
}

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
    index: HashMap<&'static str, usize>,
}

impl Default for UiConfig {
    fn default() -> Self {
        let mut config = Self {
            filter_label: "Filter facets".to_string(),
            detail_panel_title: "Selection details".to_string(),
            tabs: Vec::new(),
            index: HashMap::new(),
        };

        config.register_tab(TabUiConfig::new(
            SearchMode::FACETS,
            "Tags",
            PaneUiConfig::new(
                "Facet search",
                "Type to filter facets. Press Tab to view files.",
                "Matching facets",
                "Facets",
            ),
        ));
        config.register_tab(TabUiConfig::new(
            SearchMode::FILES,
            "Files",
            PaneUiConfig::new(
                "File search",
                "Type to filter files. Press Tab to view facets.",
                "Matching files",
                "Files",
            ),
        ));

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

        config.register_tab(TabUiConfig::new(
            SearchMode::FACETS,
            "Tags",
            PaneUiConfig::new(
                "Tag search",
                "Type to filter tags. Press Tab to view files.",
                "Matching tags",
                "Tags",
            ),
        ));

        config.register_tab(TabUiConfig::new(
            SearchMode::FILES,
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
        let id = tab.mode.as_str();
        if let Some(position) = self.index.get(id).copied() {
            self.tabs[position] = tab;
        } else {
            let idx = self.tabs.len();
            self.index.insert(id, idx);
            self.tabs.push(tab);
        }
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
            .get(mode.as_str())
            .and_then(|position| self.tabs.get(*position))
    }

    /// Lookup pane metadata for the provided mode.
    #[must_use]
    pub fn pane(&self, mode: SearchMode) -> Option<&PaneUiConfig> {
        self.tab(mode).map(|tab| &tab.pane)
    }

    /// Mutably lookup pane metadata for the provided mode.
    pub fn pane_mut(&mut self, mode: SearchMode) -> Option<&mut PaneUiConfig> {
        let position = self.index.get(mode.as_str()).copied()?;
        self.tabs.get_mut(position).map(|tab| &mut tab.pane)
    }

    /// Retrieve the label displayed on the tab itself.
    #[must_use]
    pub fn tab_label(&self, mode: SearchMode) -> Option<&str> {
        self.tab(mode).map(|tab| tab.tab_label.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_mode_uses_correct_pane() {
        let ui = UiConfig::default();
        assert_eq!(SearchMode::FACETS.title(&ui), "Facet search");
        assert_eq!(SearchMode::FILES.hint(&ui), "Type to filter files. Press Tab to view facets.");
    }
}
