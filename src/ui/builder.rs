#[cfg(feature = "fs")]
use std::sync::mpsc::Receiver;

use std::collections::HashMap;

use anyhow::Result;
#[cfg(not(feature = "fs"))]
use anyhow::bail;
use ratatui::layout::Constraint;

use super::App;
use crate::plugins::SearchPluginRegistry;
#[cfg(feature = "fs")]
use crate::systems::filesystem::{FilesystemOptions, IndexUpdate, spawn_filesystem_index};
use crate::theme::Theme;
use crate::types::{SearchData, SearchMode, SearchOutcome, UiConfig};

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
    start_mode: Option<SearchMode>,
    plugins: SearchPluginRegistry,
    #[cfg(feature = "fs")]
    index_updates: Option<Receiver<IndexUpdate>>,
}

impl SearchUi {
    /// Create a new search UI for the provided data.
    pub fn new(data: SearchData) -> Self {
        Self {
            data,
            input_title: None,
            headers: HashMap::new(),
            widths: HashMap::new(),
            ui_config: None,
            theme: None,
            start_mode: None,
            plugins: SearchPluginRegistry::default(),
            #[cfg(feature = "fs")]
            index_updates: None,
        }
    }

    /// Create a search UI pre-populated with files from the filesystem rooted at `path`.
    #[cfg(feature = "fs")]
    pub fn filesystem(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Self::filesystem_with_options(path.as_ref().to_path_buf(), FilesystemOptions::default())
    }

    #[cfg(feature = "fs")]
    pub fn filesystem_with_options(
        path: impl Into<std::path::PathBuf>,
        options: FilesystemOptions,
    ) -> Result<Self> {
        let root = path.into();
        let (data, updates) = spawn_filesystem_index(root, options)?;
        let mut ui = Self::new(data);
        ui.start_mode = Some(SearchMode::FILES);
        ui.index_updates = Some(updates);
        Ok(ui)
    }

    #[cfg(not(feature = "fs"))]
    pub fn filesystem(_path: impl AsRef<std::path::Path>) -> Result<Self> {
        bail!("filesystem support is disabled; enable the `fs` feature");
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
        if let Some(theme) = crate::theme::by_name(name) {
            self.theme = Some(theme);
        }
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn with_start_mode(mut self, mode: SearchMode) -> Self {
        self.start_mode = Some(mode);
        self
    }

    /// Replace the plugin registry driving the search worker.
    pub fn with_plugin_registry(mut self, plugins: SearchPluginRegistry) -> Self {
        self.plugins = plugins;
        self
    }

    /// Mutably configure the plugin registry.
    pub fn configure_plugins<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(&mut SearchPluginRegistry),
    {
        configure(&mut self.plugins);
        self
    }

    /// Run the interactive search UI with the configured options.
    #[cfg_attr(not(feature = "fs"), allow(unused_mut))]
    pub fn run(mut self) -> Result<SearchOutcome> {
        // Build an App and apply optional customizations, then run it.
        let mut app = App::with_plugins(self.data, self.plugins);
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
            app.set_theme(theme);
        }
        if let Some(mode) = self.start_mode {
            app.set_mode(mode);
        }
        #[cfg(feature = "fs")]
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
