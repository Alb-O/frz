use frizbee::Options;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::Paragraph;

use super::App;
use crate::logging;
use crate::systems::search;
use crate::tui::components::tables::TableSpec;
use crate::tui::components::{
	InputContext, ProgressState, TabItem, render_input_with_tabs, render_table,
};
use crate::tui::tables::rows::build_file_rows;

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
				label: tab.tab_label.as_str(),
			})
			.collect::<Vec<_>>();
		let input_ctx = InputContext {
			search_input: &self.search_input,
			input_title: self.input_title.as_deref(),
			pane_title: self.ui.pane().map(|pane| pane.mode_title.as_str()),
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
		let labels = vec![("files", "Files".to_string())];
		self.index_progress.status(&labels)
	}

	fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
		let highlight_owned = self.highlight_for_query(self.data.files.len());
		let highlight_state = highlight_owned
			.as_ref()
			.map(|(text, config)| (text.as_str(), *config));

		// Default headers and widths if not customized
		let default_headers = vec!["Path".into(), "Tags".into(), "Score".into()];
		let default_widths = vec![
			Constraint::Percentage(60),
			Constraint::Percentage(30),
			Constraint::Length(8),
		];

		let widths = self.tab_buffers.widths.as_ref().unwrap_or(&default_widths);
		let headers = self
			.tab_buffers
			.headers
			.as_ref()
			.unwrap_or(&default_headers);
		let column_widths = resolve_column_widths(area, widths);

		let rows = build_file_rows(
			&self.tab_buffers.filtered,
			&self.tab_buffers.scores,
			&self.data.files,
			highlight_state,
			self.theme.highlight_style(),
			Some(&column_widths),
		);

		let spec = TableSpec {
			headers: headers.clone(),
			widths: widths.clone(),
			rows,
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

fn resolve_column_widths(area: Rect, widths: &[Constraint]) -> Vec<u16> {
	if widths.is_empty() || area.width == 0 {
		return Vec::new();
	}

	let layout_area = Rect {
		x: 0,
		y: 0,
		width: area.width,
		height: 1,
	};
	Layout::horizontal(widths.to_vec())
		.spacing(1)
		.split(layout_area)
		.iter()
		.map(|rect| rect.width)
		.collect()
}
