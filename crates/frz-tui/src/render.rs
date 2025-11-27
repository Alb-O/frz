use frizbee::Config;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::Paragraph;

use super::App;
use super::components::rows::build_file_rows;
use super::components::tables::TableSpec;
use super::components::{
	InputContext, PreviewContext, ProgressState, render_input, render_preview, render_table,
};
use frz_core::features::search_pipeline;

impl<'a> App<'a> {
	pub(crate) fn draw(&mut self, frame: &mut Frame) {
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
		// Use tab label as placeholder
		let placeholder = self.ui.tabs().first().map(|tab| tab.tab_label.as_str());
		let input_ctx = InputContext {
			search_input: &self.search_input,
			placeholder,
			area: layout[0],
			theme: &self.style.theme,
		};
		let progress_state = ProgressState {
			progress_text: &progress_text,
			progress_complete,
			throbber_state: &self.throbber_state,
		};
		render_input(frame, input_ctx, progress_state);

		let results_area = layout[1];

		// Split horizontally if preview is enabled
		if self.preview_enabled {
			let split = Layout::default()
				.direction(Direction::Horizontal)
				.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
				.split(results_area);

			self.render_results(frame, split[0]);
			self.render_preview_pane(frame, split[1]);
		} else {
			self.render_results(frame, results_area);
		}

		if self.filtered_len() == 0 {
			let mut message_area = if self.preview_enabled {
				let split = Layout::default()
					.direction(Direction::Horizontal)
					.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
					.split(results_area);
				split[0]
			} else {
				results_area
			};
			// Account for border (1 top + 1 bottom) and header + divider (2)
			const BORDER_AND_HEADER_HEIGHT: u16 = 4;
			if message_area.height > BORDER_AND_HEADER_HEIGHT {
				// Adjust for top border
				message_area.y += 1;
				message_area.x += 1;
				message_area.width = message_area.width.saturating_sub(2);
				message_area.height -= 2; // Remove top and bottom borders

				// Now account for header and divider within the inner area
				const HEADER_AND_DIVIDER_HEIGHT: u16 = 2;
				if message_area.height > HEADER_AND_DIVIDER_HEIGHT {
					message_area.y += HEADER_AND_DIVIDER_HEIGHT;
					message_area.height -= HEADER_AND_DIVIDER_HEIGHT;

					let empty = Paragraph::new("No results").alignment(Alignment::Center);
					frame.render_widget(empty, message_area);
				}
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
			.map(|(text, config)| (text.as_str(), config.clone()));

		// Default headers and widths if not customized
		let default_headers = vec!["Path".into(), "Score".into()];
		let default_widths = vec![Constraint::Min(20), Constraint::Length(8)];

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
			self.style.theme.highlight_style(),
			Some(&column_widths),
		);

		let spec = TableSpec {
			headers: headers.clone(),
			widths: widths.clone(),
			rows,
			title: None,
		};

		render_table(frame, area, &mut self.table_state, spec, &self.style.theme);
	}

	fn render_preview_pane(&self, frame: &mut Frame, area: Rect) {
		let ctx = PreviewContext {
			content: &self.preview_content,
			scroll_offset: self.preview_scroll,
			theme: &self.style.theme,
		};
		render_preview(frame, area, ctx);
	}

	fn highlight_for_query(&self, dataset_len: usize) -> Option<(String, Config)> {
		let query = self.search_input.text().trim();
		if query.is_empty() {
			return None;
		}
		let config = search_pipeline::config_for_query(query, dataset_len);
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
