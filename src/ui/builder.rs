use std::sync::mpsc::Receiver;

use anyhow::Result;
use ratatui::layout::Constraint;

use super::App;
use super::config::UiConfig;
use crate::search::{SearchData, SearchOutcome};
use crate::systems::filesystem::{FilesystemOptions, IndexResult, spawn_filesystem_index};
pub use crate::ui::style::Theme;

/// A small builder for configuring the interactive search UI.
/// This presents an fzf-like API for setting prompts, column
/// headings and column widths before running the interactive picker.
pub struct SearchUi {
	data: SearchData,
	input_title: Option<String>,
	headers: Option<Vec<String>>,
	widths: Option<Vec<Constraint>>,
	ui_config: Option<UiConfig>,
	theme: Option<Theme>,
	bat_theme: Option<String>,
	index_updates: Option<Receiver<IndexResult>>,
	preview_enabled: bool,
}

impl SearchUi {
	/// Create a new search UI for the provided data.
	pub fn new(data: SearchData) -> Self {
		Self {
			data,
			input_title: None,
			headers: None,
			widths: None,
			ui_config: None,
			theme: None,
			bat_theme: None,
			index_updates: None,
			preview_enabled: false,
		}
	}

	/// Create a search UI pre-populated with files from the filesystem rooted at `path`.
	pub fn filesystem(path: impl AsRef<std::path::Path>) -> Result<Self> {
		Self::filesystem_with_options(path.as_ref().to_path_buf(), FilesystemOptions::default())
	}

	/// Create a search UI with custom filesystem scanning options.
	pub fn filesystem_with_options(
		path: impl Into<std::path::PathBuf>,
		options: FilesystemOptions,
	) -> Result<Self> {
		let root = path.into();
		let (data, updates) = spawn_filesystem_index(root, options)?;
		let mut ui = Self::new(data);
		ui.index_updates = Some(updates);
		Ok(ui)
	}

	/// Set the title displayed above the filter input.
	pub fn with_input_title(mut self, title: impl Into<String>) -> Self {
		self.input_title = Some(title.into());
		self
	}

	/// Set column headers for the results table.
	pub fn with_headers(mut self, headers: Vec<&str>) -> Self {
		self.headers = Some(headers.into_iter().map(|s| s.to_string()).collect());
		self
	}

	/// Set column widths for the results table.
	pub fn with_widths(mut self, widths: Vec<Constraint>) -> Self {
		self.widths = Some(widths);
		self
	}

	/// Apply a complete UI configuration override.
	pub fn with_ui_config(mut self, config: UiConfig) -> Self {
		self.ui_config = Some(config);
		self
	}

	/// Pre-populate the filter input with an initial query.
	pub fn with_initial_query(mut self, query: impl Into<String>) -> Self {
		self.data.initial_query = query.into();
		self
	}

	/// Select a theme by name.
	pub fn with_theme_name(mut self, name: &str) -> Self {
		if let Some(theme) = crate::ui::style::by_name(name) {
			self.theme = Some(theme);
			self.bat_theme = crate::ui::style::bat_theme(name);
		}
		self
	}

	/// Set a custom theme.
	pub fn with_theme(mut self, theme: Theme) -> Self {
		self.theme = Some(theme);
		self.bat_theme = None;
		self
	}

	/// Enable the preview pane by default when the UI starts.
	pub fn with_preview(mut self) -> Self {
		self.preview_enabled = true;
		self
	}

	/// Run the interactive search UI with the configured options.
	pub fn run(mut self) -> Result<SearchOutcome> {
		// Build an App and apply optional customizations, then run it.
		let mut app = App::new(self.data);
		if let Some(title) = self.input_title {
			app.input_title = Some(title);
		}
		if let Some(headers) = self.headers {
			app.set_headers(headers);
		}
		if let Some(widths) = self.widths {
			app.set_widths(widths);
		}
		if let Some(ui) = self.ui_config {
			app.ui = ui;
			app.ensure_tab_buffers();
		}
		if let Some(theme) = self.theme {
			app.set_theme_with_bat(theme, self.bat_theme.clone());
		}
		if let Some(updates) = self.index_updates.take() {
			app.set_index_updates(updates);
		}
		if self.preview_enabled {
			app.enable_preview();
		}

		app.run()
	}
}
