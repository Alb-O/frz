#[cfg(feature = "fs")]
use std::sync::mpsc::Receiver;

use anyhow::Result;
#[cfg(not(feature = "fs"))]
use anyhow::bail;
use ratatui::layout::Constraint;

use crate::app::App;
#[cfg(feature = "fs")]
use crate::indexing::{FilesystemOptions, IndexUpdate, spawn_filesystem_index};
use crate::theme::Theme;
use crate::types::{SearchData, SearchMode, SearchOutcome, UiConfig};

/// A small builder for configuring the TUI searcher.
/// This presents an fzf-like API for setting prompts, column
/// headings and column widths before running the interactive picker.
pub struct Searcher {
    data: SearchData,
    input_title: Option<String>,
    facet_headers: Option<Vec<String>>,
    file_headers: Option<Vec<String>>,
    facet_widths: Option<Vec<Constraint>>,
    file_widths: Option<Vec<Constraint>>,
    ui_config: Option<UiConfig>,
    theme: Option<Theme>,
    start_mode: Option<SearchMode>,
    #[cfg(feature = "fs")]
    index_updates: Option<Receiver<IndexUpdate>>,
}

impl Searcher {
    /// Create a new Searcher for the provided data.
    pub fn new(data: SearchData) -> Self {
        Self {
            data,
            input_title: None,
            facet_headers: None,
            file_headers: None,
            facet_widths: None,
            file_widths: None,
            ui_config: None,
            theme: None,
            start_mode: None,
            #[cfg(feature = "fs")]
            index_updates: None,
        }
    }

    /// Create a searcher pre-populated with files from the filesystem rooted at `path`.
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
        let mut searcher = Self::new(data);
        searcher.start_mode = Some(SearchMode::Files);
        searcher.index_updates = Some(updates);
        Ok(searcher)
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
        match mode {
            SearchMode::Facets => self.facet_widths = Some(widths),
            SearchMode::Files => self.file_widths = Some(widths),
        }
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

    /// Run the interactive searcher with the configured options.
    #[cfg_attr(not(feature = "fs"), allow(unused_mut))]
    pub fn run(mut self) -> Result<SearchOutcome> {
        // Build an App and apply optional customizations, then run it.
        let mut app = App::new(self.data);
        if let Some(title) = self.input_title {
            app.input_title = Some(title);
        }
        if let Some(headers) = self.facet_headers {
            app.facet_headers = Some(headers);
        }
        if let Some(headers) = self.file_headers {
            app.file_headers = Some(headers);
        }
        if let Some(widths) = self.facet_widths {
            app.facet_widths = Some(widths);
        }
        if let Some(widths) = self.file_widths {
            app.file_widths = Some(widths);
        }
        if let Some(ui) = self.ui_config {
            app.ui = ui;
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
        match mode {
            SearchMode::Facets => self.facet_headers = Some(headers),
            SearchMode::Files => self.file_headers = Some(headers),
        }
        self
    }
}
