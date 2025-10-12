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

/// Text used by the UI when rendering facet and file panes.
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub filter_label: String,
    pub facets: PaneUiConfig,
    pub files: PaneUiConfig,
    pub detail_panel_title: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            filter_label: "Filter facets".to_string(),
            facets: PaneUiConfig::new(
                "Facet search",
                "Type to filter facets. Press Tab to view files.",
                "Matching facets",
                "Facets",
            ),
            files: PaneUiConfig::new(
                "File search",
                "Type to filter files. Press Tab to view facets.",
                "Matching files",
                "Files",
            ),
            detail_panel_title: "Selection details".to_string(),
        }
    }
}

impl UiConfig {
    /// UI configuration tailored to searching tags and files.
    #[must_use]
    pub fn tags_and_files() -> Self {
        Self {
            filter_label: "Filter tag".to_string(),
            facets: PaneUiConfig::new(
                "Tag search",
                "Type to filter tags. Press Tab to view files.",
                "Matching tags",
                "Tags",
            ),
            files: PaneUiConfig::new(
                "File search",
                "Type to filter files by path or tag. Press Tab to view tags.",
                "Matching files",
                "Files",
            ),
            detail_panel_title: "Selection details".to_string(),
        }
    }
}

/// Indicates whether the UI is focusing on facets or files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Facets,
    Files,
}

impl SearchMode {
    /// Return the active pane title for the current mode.
    pub fn title(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.mode_title.as_str(),
            SearchMode::Files => ui.files.mode_title.as_str(),
        }
    }

    /// Return the hint text for the current mode.
    pub fn hint(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.hint.as_str(),
            SearchMode::Files => ui.files.hint.as_str(),
        }
    }

    /// Return the table title for the current mode.
    pub fn table_title(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.table_title.as_str(),
            SearchMode::Files => ui.files.table_title.as_str(),
        }
    }

    /// Return the count label for the current mode.
    #[must_use]
    pub fn count_label(self, ui: &UiConfig) -> &str {
        match self {
            SearchMode::Facets => ui.facets.count_label.as_str(),
            SearchMode::Files => ui.files.count_label.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_mode_uses_correct_pane() {
        let ui = UiConfig::default();
        assert_eq!(SearchMode::Facets.title(&ui), "Facet search");
        assert_eq!(
            SearchMode::Files.hint(&ui),
            "Type to filter files. Press Tab to view facets."
        );
    }
}
