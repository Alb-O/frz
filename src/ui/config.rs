use std::collections::HashMap;

use crate::extensions::api::SearchMode;

/// Human-readable labels and titles rendered within a single search pane.
#[derive(Debug, Clone)]
pub struct PaneUiConfig {
	/// Title shown above the pane when it is active.
	pub mode_title: String,
	/// Inline hint displayed beneath the pane title.
	pub hint: String,
	/// Title rendered above the table of results.
	pub table_title: String,
	/// Label summarizing the number of visible entries.
	pub count_label: String,
}

impl PaneUiConfig {
	/// Construct a new [`PaneUiConfig`] from the individual bits of text shown
	/// alongside the pane.
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

/// Complete UI definition for a contributed tab and its associated pane.
#[derive(Debug, Clone)]
pub struct TabUiConfig {
	/// Identifier used to look up results for this tab.
	pub mode: SearchMode,
	/// Label rendered on the tab selector.
	pub tab_label: String,
	/// Text displayed within the tab's primary pane.
	pub pane: PaneUiConfig,
}

impl TabUiConfig {
	/// Build a [`TabUiConfig`] by combining the mode identifier, tab label, and
	/// pane configuration.
	#[must_use]
	pub fn new(mode: SearchMode, tab_label: impl Into<String>, pane: PaneUiConfig) -> Self {
		Self {
			mode,
			tab_label: tab_label.into(),
			pane,
		}
	}
}

/// Textual configuration used when rendering panes, tabs, and surrounding UI.
#[derive(Debug, Clone)]
pub struct UiConfig {
	/// Placeholder text displayed next to the filter input.
	pub filter_label: String,
	/// Title used for the detail panel.
	pub detail_panel_title: String,
	tabs: Vec<TabUiConfig>,
	index: HashMap<SearchMode, usize>,
}

impl Default for UiConfig {
	fn default() -> Self {
		let mut config = Self {
			filter_label: "Filter attributes".to_string(),
			detail_panel_title: "Selection details".to_string(),
			tabs: Vec::new(),
			index: HashMap::new(),
		};
		for meta in crate::extensions::builtin::all_metadata() {
			let pane = PaneUiConfig::new(
				meta.mode_title,
				meta.hint,
				meta.table_title,
				meta.count_label,
			);
			config.register_tab(TabUiConfig::new(meta.mode, meta.tab_label, pane));
		}
		config
	}
}

impl UiConfig {
	/// Build a configuration with built-in panes for file searching.
	#[must_use]
	pub fn tags_and_files() -> Self {
		let mut config = Self {
			filter_label: "Filter files".to_string(),
			detail_panel_title: "Selection details".to_string(),
			tabs: Vec::new(),
			index: HashMap::new(),
		};

		let files = crate::extensions::builtin::files::mode();
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

	/// Register a new tab definition with this configuration, replacing an
	/// existing tab for the same [`SearchMode`] if necessary.
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

	/// Return all registered tabs in the order they were added, preserving the
	/// explicit registration order when extensions are loaded.
	#[must_use]
	pub fn tabs(&self) -> &[TabUiConfig] {
		&self.tabs
	}

	/// Look up tab metadata for the provided mode, if it has been registered.
	#[must_use]
	pub fn tab(&self, mode: SearchMode) -> Option<&TabUiConfig> {
		self.index
			.get(&mode)
			.and_then(|position| self.tabs.get(*position))
	}

	/// Look up pane metadata for the provided mode, if it has been registered.
	#[must_use]
	pub fn pane(&self, mode: SearchMode) -> Option<&PaneUiConfig> {
		self.tab(mode).map(|tab| &tab.pane)
	}

	/// Mutably look up pane metadata for the provided mode, if it has been registered.
	pub fn pane_mut(&mut self, mode: SearchMode) -> Option<&mut PaneUiConfig> {
		let position = self.index.get(&mode).copied()?;
		self.tabs.get_mut(position).map(|tab| &mut tab.pane)
	}

	/// Retrieve the label displayed on the tab itself for the provided mode.
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

	/// Return the pane title associated with the provided mode, defaulting to an
	/// empty string when the mode is unknown.
	#[must_use]
	pub fn mode_title(&self, mode: SearchMode) -> &str {
		self.pane(mode)
			.map(|pane| pane.mode_title.as_str())
			.unwrap_or("")
	}

	/// Return the hint text associated with the provided mode, defaulting to an
	/// empty string when the mode is unknown.
	#[must_use]
	pub fn mode_hint(&self, mode: SearchMode) -> &str {
		self.pane(mode).map(|pane| pane.hint.as_str()).unwrap_or("")
	}

	/// Return the table title associated with the provided mode, defaulting to
	/// an empty string when the mode is unknown.
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
	use crate::extensions::builtin::files;

	#[test]
	fn tags_and_files_registers_tabs() {
		let config = UiConfig::tags_and_files();
		assert!(config.tab(files::mode()).is_some());
	}
}
