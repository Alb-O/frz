use anyhow::Result;
use frz::plugins::builtin::{attributes, files};
use frz::{FilesystemOptions, SearchMode, SearchOutcome, SearchUi, UiConfig};
use std::path::PathBuf;

use crate::settings::ResolvedConfig;

/// Coordinates building and running the interactive search experience.
pub(crate) struct SearchWorkflow {
    search_ui: SearchUi,
}

impl SearchWorkflow {
    pub(crate) fn from_config(config: ResolvedConfig) -> Result<Self> {
        let search_ui = SearchUiFactory::build(config)?;
        Ok(Self { search_ui })
    }

    pub(crate) fn run(self) -> Result<SearchOutcome> {
        self.search_ui.run()
    }
}

/// Helper for translating resolved configuration into a configured `SearchUi`.
struct SearchUiFactory {
    search_ui: SearchUi,
}

impl SearchUiFactory {
    fn build(config: ResolvedConfig) -> Result<SearchUi> {
        let ResolvedConfig {
            root,
            filesystem,
            input_title,
            initial_query,
            theme,
            start_mode,
            ui,
            facet_headers,
            file_headers,
        } = config;

        let builder = Self::new(root, filesystem)?
            .with_input_title(input_title)
            .with_ui_config(ui)
            .with_initial_query(initial_query)
            .with_theme(theme)
            .with_start_mode(start_mode)
            .with_headers(attributes::mode(), facet_headers)
            .with_headers(files::mode(), file_headers);

        Ok(builder.finish())
    }

    fn new(root: PathBuf, filesystem: FilesystemOptions) -> Result<Self> {
        let search_ui = SearchUi::filesystem_with_options(root, filesystem)?;
        Ok(Self { search_ui })
    }

    fn with_input_title(mut self, title: Option<String>) -> Self {
        if let Some(title) = title {
            self.search_ui = self.search_ui.with_input_title(title);
        }
        self
    }

    fn with_ui_config(mut self, config: UiConfig) -> Self {
        self.search_ui = self.search_ui.with_ui_config(config);
        self
    }

    fn with_initial_query(mut self, query: String) -> Self {
        self.search_ui = self.search_ui.with_initial_query(query);
        self
    }

    fn with_theme(mut self, theme: Option<String>) -> Self {
        if let Some(theme) = theme {
            self.search_ui = self.search_ui.with_theme_name(&theme);
        }
        self
    }

    fn with_start_mode(mut self, mode: Option<SearchMode>) -> Self {
        if let Some(mode) = mode {
            self.search_ui = self.search_ui.with_start_mode(mode);
        }
        self
    }

    fn with_headers(mut self, mode: SearchMode, headers: Option<Vec<String>>) -> Self {
        if let Some(headers) = headers {
            let refs: Vec<&str> = headers.iter().map(|header| header.as_str()).collect();
            self.search_ui = self.search_ui.with_headers_for(mode, refs);
        }
        self
    }

    fn finish(self) -> SearchUi {
        self.search_ui
    }
}
