use anyhow::Result;
use frz_core::{SearchOutcome, SearchUi};

use crate::config::Config;

/// Coordinates building and running the interactive search experience.
pub(crate) struct SearchWorkflow {
	search_ui: SearchUi,
}

impl SearchWorkflow {
	/// Build workflow from configuration, applying UI settings and initial state.
	pub(crate) fn from_config(config: Config) -> Result<Self> {
		let Config {
			root,
			filesystem,
			input_title,
			initial_query,
			theme,
			ui,
			file_headers,
		} = config;

		let mut search_ui = SearchUi::filesystem_with_options(root, filesystem)?;

		if let Some(title) = input_title {
			search_ui = search_ui.with_input_title(title);
		}

		search_ui = search_ui.with_ui_config(ui);
		search_ui = search_ui.with_initial_query(initial_query);

		if let Some(theme) = theme {
			search_ui = search_ui.with_theme_name(&theme);
		}

		if let Some(headers) = file_headers {
			let refs: Vec<&str> = headers.iter().map(|h| h.as_str()).collect();
			search_ui = search_ui.with_headers(refs);
		}

		Ok(Self { search_ui })
	}

	/// Run the interactive search UI and return the final outcome.
	pub(crate) fn run(self) -> Result<SearchOutcome> {
		self.search_ui.run()
	}
}
