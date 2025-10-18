use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use anyhow::Result;
use ratatui::layout::Constraint;

use super::App;
use super::config::UiConfig;
use crate::extensions::api::{ExtensionCatalog, SearchData, SearchMode, SearchOutcome};
use crate::systems::filesystem::{FilesystemOptions, IndexUpdate, spawn_filesystem_index};
pub use crate::tui::theme::Theme;

/// A small builder for configuring the interactive search UI.
/// This presents an fzf-like API for setting prompts, column
/// headings and column widths before running the interactive picker.
pub struct SearchUi {
    data: SearchData,
    input_title: Option<String>,
    headers: HashMap<SearchMode, Vec<String>>,
    widths: HashMap<SearchMode, Vec<Constraint>>,
    ui_config: Option<UiConfig>,
    theme: Option<Theme>,
    bat_theme: Option<String>,
    start_mode: Option<SearchMode>,
    extensions: ExtensionCatalog,
    index_updates: Option<Receiver<IndexUpdate>>,
}

impl SearchUi {
    /// Create a new search UI for the provided data.
    pub fn new(data: SearchData) -> Self {
        let mut extensions = ExtensionCatalog::default();
        crate::extensions::builtin::register_builtin_extensions(&mut extensions)
            .expect("builtin extensions must register successfully");

        Self {
            data,
            input_title: None,
            headers: HashMap::new(),
            widths: HashMap::new(),
            ui_config: None,
            theme: None,
            bat_theme: None,
            start_mode: None,
            extensions,
            index_updates: None,
        }
    }

    /// Create a search UI pre-populated with files from the filesystem rooted at `path`.
    pub fn filesystem(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Self::filesystem_with_options(path.as_ref().to_path_buf(), FilesystemOptions::default())
    }

    pub fn filesystem_with_options(
        path: impl Into<std::path::PathBuf>,
        options: FilesystemOptions,
    ) -> Result<Self> {
        let root = path.into();
        let (data, updates) = spawn_filesystem_index(root, options)?;
        let mut ui = Self::new(data);
        ui.start_mode = Some(crate::extensions::builtin::files::mode());
        ui.index_updates = Some(updates);
        Ok(ui)
    }

    pub fn with_input_title(mut self, title: impl Into<String>) -> Self {
        self.input_title = Some(title.into());
        self
    }

    pub fn with_headers_for(self, mode: SearchMode, headers: Vec<&str>) -> Self {
        let headers = headers.into_iter().map(|s| s.to_string()).collect();
        self.with_headers(mode, headers)
    }

    pub fn with_widths_for(mut self, mode: SearchMode, widths: Vec<Constraint>) -> Self {
        self.widths.insert(mode, widths);
        self
    }

    pub fn with_ui_config(mut self, config: UiConfig) -> Self {
        self.ui_config = Some(config);
        self
    }

    pub fn with_initial_query(mut self, query: impl Into<String>) -> Self {
        self.data.initial_query = query.into();
        self
    }

    pub fn with_theme_name(mut self, name: &str) -> Self {
        if let Some(theme) = crate::tui::theme::by_name(name) {
            self.theme = Some(theme);
            self.bat_theme = crate::tui::theme::bat_theme(name);
        }
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self.bat_theme = None;
        self
    }

    pub fn with_start_mode(mut self, mode: SearchMode) -> Self {
        self.start_mode = Some(mode);
        self
    }

    /// Replace the extension catalog driving the search worker.
    pub fn with_extension_catalog(mut self, extensions: ExtensionCatalog) -> Self {
        self.extensions = extensions;
        self
    }

    /// Mutably configure the extension catalog.
    pub fn configure_extensions<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(&mut ExtensionCatalog),
    {
        configure(&mut self.extensions);
        self
    }

    /// Run the interactive search UI with the configured options.
    pub fn run(mut self) -> Result<SearchOutcome> {
        // Build an App and apply optional customizations, then run it.
        let mut app = App::with_extensions(self.data, self.extensions);
        if let Some(title) = self.input_title {
            app.input_title = Some(title);
        }
        for (mode, headers) in self.headers {
            app.set_headers_for(mode, headers);
        }
        for (mode, widths) in self.widths {
            app.set_widths_for(mode, widths);
        }
        if let Some(ui) = self.ui_config {
            app.ui = ui;
            app.ensure_tab_buffers();
        }
        if let Some(theme) = self.theme {
            app.set_theme_with_bat(theme, self.bat_theme.clone());
        }
        if let Some(mode) = self.start_mode {
            app.set_mode(mode);
        }
        if let Some(updates) = self.index_updates.take() {
            app.set_index_updates(updates);
        }

        app.run()
    }

    fn with_headers(mut self, mode: SearchMode, headers: Vec<String>) -> Self {
        self.headers.insert(mode, headers);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::builtin::{attributes, files};

    #[test]
    fn builder_registers_builtin_extensions() {
        let ui = SearchUi::new(SearchData::new());

        assert!(
            ui.extensions.contains_mode(attributes::mode()),
            "expected attributes extension to be registered"
        );
        assert!(
            ui.extensions.contains_mode(files::mode()),
            "expected files extension to be registered"
        );
    }
}
