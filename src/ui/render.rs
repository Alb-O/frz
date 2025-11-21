use frizbee::Options;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::widgets::Paragraph;

use super::App;
use crate::extensions::api::SearchMode;
use crate::extensions::builtin::files;
use crate::logging;
use crate::systems::search;
use crate::tui::components::{
	InputContext, ProgressState, TabItem, TableRenderContext, render_input_with_tabs, render_table,
};

impl<'a> App<'a> {
	pub(crate) fn draw(&mut self, frame: &mut Frame) {
		logging::pump();

		let area = frame.area();
		let area = area.inner(Margin {
			vertical: 0,
			horizontal: 1,
		});

		let layout = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(1), Constraint::Min(1)])
			.split(area);

		let (progress_text, progress_complete) = self.progress_status();
		let tabs = self
			.ui
			.tabs()
			.iter()
			.map(|tab| TabItem {
				mode: tab.mode,
				label: tab.tab_label.as_str(),
			})
			.collect::<Vec<_>>();
		let input_ctx = InputContext {
			search_input: &self.search_input,
			input_title: self.input_title.as_deref(),
			pane_title: self.ui.pane(self.mode).map(|pane| pane.mode_title.as_str()),
			mode: self.mode,
			tabs: &tabs,
			area: layout[0],
			theme: &self.theme,
		};
		let progress_state = ProgressState {
			progress_text: &progress_text,
			progress_complete,
			throbber_state: &self.throbber_state,
		};
		render_input_with_tabs(frame, input_ctx, progress_state);
		let results_area = layout[1];
		self.render_results(frame, results_area);

		if self.filtered_len() == 0 {
			let mut message_area = results_area;
			const HEADER_AND_DIVIDER_HEIGHT: u16 = 2;
			if message_area.height > HEADER_AND_DIVIDER_HEIGHT {
				message_area.y += HEADER_AND_DIVIDER_HEIGHT;
				message_area.height -= HEADER_AND_DIVIDER_HEIGHT;

				let empty = Paragraph::new("No results").alignment(Alignment::Center);
				frame.render_widget(empty, message_area);
			}
		}
	}

	fn progress_status(&mut self) -> (String, bool) {
		let mut labels = Vec::new();
		for tab in self.ui.tabs() {
			labels.push((tab.mode.id(), tab.pane.count_label.clone()));
		}
		self.index_progress.status(&labels)
	}

	fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
		let highlight_owned = self.highlight_for_query(dataset_len(&self.data, self.mode));
		let highlight_state = highlight_owned
			.as_ref()
			.map(|(text, config)| (text.as_str(), *config));
		let state = self.tab_states.entry(self.mode).or_default();

		let spec = match self.mode {
			SearchMode::Files => files::table(
				TableRenderContext {
					area,
					filtered: &state.filtered,
					scores: &state.scores,
					headers: state.headers.as_ref(),
					widths: state.widths.as_ref(),
					highlight: highlight_state,
					data: &self.data,
				},
				&self.theme,
			),
		};

		render_table(frame, area, &mut self.table_state, spec, &self.theme);
	}

	fn highlight_for_query(&self, dataset_len: usize) -> Option<(String, Options)> {
		let query = self.search_input.text().trim();
		if query.is_empty() {
			return None;
		}
		let config = search::config_for_query(query, dataset_len);
		Some((query.to_string(), config))
	}
}

fn dataset_len(data: &crate::extensions::api::SearchData, mode: SearchMode) -> usize {
	match mode {
		SearchMode::Files => data.files.len(),
	}
}
