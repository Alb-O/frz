use anyhow::Result;
use frz::UiConfig;
use frz::extensions::builtin::files;
use serde::Deserialize;

use super::super::ui::{apply_pane_config, ui_from_preset};
use super::super::util::sanitize_headers;
use crate::cli::CliArgs;

/// UI related configuration values prior to validation.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(super) struct UiSection {
	pub(super) input_title: Option<String>,
	pub(super) initial_query: Option<String>,
	pub(super) theme: Option<String>,
	pub(super) preset: Option<String>,
	pub(super) filter_label: Option<String>,
	pub(super) detail_panel_title: Option<String>,
	pub(super) files: Option<PaneSection>,
	pub(super) file_headers: Option<Vec<String>>,
}

/// Raw configuration for a specific UI pane.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct PaneSection {
	pub(crate) mode_title: Option<String>,
	pub(crate) hint: Option<String>,
	pub(crate) table_title: Option<String>,
	pub(crate) count_label: Option<String>,
}

pub(super) struct UiResolution {
	pub(super) ui: UiConfig,
	pub(super) input_title: Option<String>,
	pub(super) initial_query: String,
	pub(super) theme: Option<String>,
	pub(super) file_headers: Option<Vec<String>>,
}

impl UiSection {
	pub(super) fn apply_cli_overrides(&mut self, cli: &CliArgs) {
		if let Some(title) = cli.title.clone() {
			self.input_title = Some(title);
		}
		if let Some(query) = cli.initial_query.clone() {
			self.initial_query = Some(query);
		}
		if let Some(theme) = cli.theme.clone() {
			self.theme = Some(theme);
		}
		if let Some(preset) = cli.ui_preset {
			self.preset = Some(preset.as_str().to_string());
		}
		if let Some(label) = cli.filter_label.clone() {
			self.filter_label = Some(label);
		}
		if let Some(label) = cli.detail_title.clone() {
			self.detail_panel_title = Some(label);
		}
		if let Some(value) = cli.files_mode_title.clone() {
			let files = self.files.get_or_insert_with(PaneSection::default);
			files.mode_title = Some(value);
		}
		if let Some(value) = cli.files_hint.clone() {
			let files = self.files.get_or_insert_with(PaneSection::default);
			files.hint = Some(value);
		}
		if let Some(value) = cli.files_table_title.clone() {
			let files = self.files.get_or_insert_with(PaneSection::default);
			files.table_title = Some(value);
		}
		if let Some(value) = cli.files_count_label.clone() {
			let files = self.files.get_or_insert_with(PaneSection::default);
			files.count_label = Some(value);
		}
		if let Some(headers) = &cli.file_headers {
			self.file_headers = Some(headers.clone());
		}
	}

	pub(super) fn finalize(self, default_title: String) -> Result<UiResolution> {
		let mut ui = ui_from_preset(self.preset.as_deref())?;
		if let Some(label) = self.filter_label {
			ui.filter_label = label;
		}
		if let Some(detail) = self.detail_panel_title {
			ui.detail_panel_title = detail;
		}
		if let Some(pane) = self.files
			&& let Some(mode) = ui.mode_by_id(files::DATASET_KEY)
			&& let Some(target) = ui.pane_mut(mode)
		{
			apply_pane_config(target, pane);
		}

		let file_headers = self
			.file_headers
			.map(sanitize_headers)
			.filter(|headers| !headers.is_empty());

		let input_title = match self.input_title {
			Some(title) => Some(title),
			None => Some(default_title),
		};
		let initial_query = self.initial_query.unwrap_or_default();
		let theme = self.theme;

		Ok(UiResolution {
			ui,
			input_title,
			initial_query,
			theme,
			file_headers,
		})
	}
}
