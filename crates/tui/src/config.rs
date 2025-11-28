/// Human-readable labels and titles rendered within a single search pane.
#[derive(Debug, Clone)]
pub struct PaneLabels {
	/// Title shown above the pane when it is active.
	pub mode_title: String,
	/// Inline hint displayed beneath the pane title.
	pub hint: String,
	/// Title rendered above the table of results.
	pub table_title: String,
	/// Label summarizing the number of visible entries.
	pub count_label: String,
}

impl PaneLabels {
	/// Construct a new [`PaneLabels`] from the individual bits of text shown
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
pub struct TabLabels {
	/// Label rendered on the tab selector.
	pub tab_label: String,
	/// Text displayed within the tab's primary pane.
	pub pane: PaneLabels,
}

impl TabLabels {
	/// Build a [`TabLabels`] by combining the tab label and pane configuration.
	#[must_use]
	pub fn new(tab_label: impl Into<String>, pane: PaneLabels) -> Self {
		Self {
			tab_label: tab_label.into(),
			pane,
		}
	}
}

/// Textual configuration used when rendering panes, tabs, and surrounding UI.
#[derive(Debug, Clone)]
pub struct UiLabels {
	/// Placeholder text displayed next to the filter input.
	pub filter_label: String,
	/// Title used for the detail panel.
	pub detail_panel_title: String,
	tabs: Vec<TabLabels>,
}

impl Default for UiLabels {
	fn default() -> Self {
		let mut config = Self {
			filter_label: "Filter files".to_string(),
			detail_panel_title: "Selection details".to_string(),
			tabs: Vec::new(),
		};
		let pane = PaneLabels::new(
			"File search",
			"Type to filter files by path.",
			"Matching files",
			"Files",
		);
		config.register_tab(TabLabels::new("Files", pane));
		config
	}
}

impl UiLabels {
	/// Register a new tab definition with this configuration.
	pub fn register_tab(&mut self, tab: TabLabels) {
		self.tabs.push(tab);
	}

	/// Return all registered tabs in the order they were added.
	#[must_use]
	pub fn tabs(&self) -> &[TabLabels] {
		&self.tabs
	}

	/// Look up pane metadata for the single tab.
	#[must_use]
	pub fn pane(&self) -> Option<&PaneLabels> {
		self.tabs.first().map(|tab| &tab.pane)
	}

	/// Mutably look up pane metadata for the single tab.
	pub fn pane_mut(&mut self) -> Option<&mut PaneLabels> {
		self.tabs.first_mut().map(|tab| &mut tab.pane)
	}

	/// Return the pane title, defaulting to an empty string when unavailable.
	#[must_use]
	pub fn mode_title(&self) -> &str {
		self.pane()
			.map(|pane| pane.mode_title.as_str())
			.unwrap_or("")
	}

	/// Return the hint text, defaulting to an empty string when unavailable.
	#[must_use]
	pub fn mode_hint(&self) -> &str {
		self.pane().map(|pane| pane.hint.as_str()).unwrap_or("")
	}

	/// Return the table title, defaulting to an empty string when unavailable.
	#[must_use]
	pub fn mode_table_title(&self) -> &str {
		self.pane()
			.map(|pane| pane.table_title.as_str())
			.unwrap_or("")
	}
}
