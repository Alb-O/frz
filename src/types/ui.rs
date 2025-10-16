use std::collections::HashMap;

use crate::plugins::descriptors::{SearchPluginDescriptor, SearchPluginUiDefinition};

/// Identifies a single tab contributed to the search UI.
#[derive(Clone, Copy)]
pub struct SearchMode {
    descriptor: &'static SearchPluginDescriptor,
}

impl std::fmt::Debug for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SearchMode").field(&self.id()).finish()
    }
}

impl PartialEq for SearchMode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.descriptor, other.descriptor)
    }
}

impl Eq for SearchMode {}

impl std::hash::Hash for SearchMode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&(self.descriptor as *const SearchPluginDescriptor), state);
    }
}

impl SearchMode {
    /// Create a search mode identifier backed by a plugin descriptor.
    #[must_use]
    pub const fn from_descriptor(descriptor: &'static SearchPluginDescriptor) -> Self {
        Self { descriptor }
    }

    /// Return the identifier for this mode.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.descriptor.id
    }

    /// Access the plugin descriptor backing this mode.
    #[must_use]
    pub const fn descriptor(self) -> &'static SearchPluginDescriptor {
        self.descriptor
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
        ui.pane(self).map(|pane| pane.hint.as_str()).unwrap_or("")
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
        let SearchPluginUiDefinition {
            tab_label,
            mode_title,
            hint,
            table_title,
            count_label,
        } = descriptor.ui;
        self.register_tab(TabUiConfig::new(
            mode,
            tab_label,
            PaneUiConfig::new(mode_title, hint, table_title, count_label),
        ));
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::builtin::{facets, files};

    #[test]
    fn search_mode_uses_correct_pane() {
        let ui = UiConfig::default();
        assert_eq!(facets::mode().title(&ui), "Facet search");
        assert_eq!(
            files::mode().hint(&ui),
            "Type to filter files. Press Tab to view facets."
        );
    }
}
