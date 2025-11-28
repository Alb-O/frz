pub(crate) mod layout;

use frizbee::Config;
use frz_core::search_pipeline;
use layout::resolve_column_widths;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::Paragraph;

use super::App;
use crate::components::rows::build_file_rows;
use crate::components::tables::{TABLE_HIGHLIGHT_SPACING, TableSpec};
use crate::components::{
	InputContext, PreviewContext, ProgressState, render_input, render_preview, render_table,
};

impl App<'_> {
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
		if self.preview.enabled {
			let split = Layout::default()
				.direction(Direction::Horizontal)
				.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
				.split(results_area);

			self.results.area = Some(split[0]);
			self.preview.area = Some(split[1]);
			self.render_results(frame, split[0]);
			self.render_preview_pane(frame, split[1]);
		} else {
			self.preview.area = None;
			self.preview.hovered = false;
			self.results.dragging = false;
			self.results.area = Some(results_area);
			self.render_results(frame, results_area);
		}

		if self.filtered_len() == 0 {
			let mut message_area = if self.preview.enabled {
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

				// Account for header and divider within the inner area
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
		// Update scrollbar state based on current viewport
		let inner_height = area.height.saturating_sub(2) as usize;
		self.results.update_scrollbar(inner_height);

		let highlight_owned = self.highlight_for_query(self.data.files.len());
		let highlight_state = highlight_owned
			.as_ref()
			.map(|(text, config)| (text.as_str(), config.clone()));

		// Default headers and widths if not set
		let default_headers = vec!["Path".into(), "Score".into()];
		let default_widths = vec![Constraint::Min(20), Constraint::Length(8)];

		let widths = self
			.results
			.buffers
			.widths
			.as_ref()
			.unwrap_or(&default_widths);
		let headers = self
			.results
			.buffers
			.headers
			.as_ref()
			.unwrap_or(&default_headers);
		let has_selection = self.results.table_state.selected().is_some();
		let column_widths = resolve_column_widths(area, widths, has_selection);

		let rows = build_file_rows(
			&self.results.buffers.filtered,
			&self.results.buffers.scores,
			&self.data.files,
			highlight_state,
			self.style.theme.highlight,
			Some(&column_widths),
		);

		let spec = TableSpec {
			headers: headers.clone(),
			widths: widths.clone(),
			rows,
			title: None,
			highlight_spacing: TABLE_HIGHLIGHT_SPACING,
		};

		render_table(
			frame,
			area,
			&mut self.results.table_state,
			&mut self.results.scrollbar_state,
			&mut self.results.scrollbar_area,
			spec,
			&self.style.theme,
		);
	}

	fn render_preview_pane(&mut self, frame: &mut Frame, area: Rect) {
		// Update viewport height (accounting for borders)
		self.preview.viewport_height = area.height.saturating_sub(2) as usize;
		let inner_width = area.width.saturating_sub(2) as usize;
		let wrap_width = inner_width.saturating_sub(1);
		self.rebuild_preview_wrap(wrap_width);
		self.update_scrollbar_state();

		let ctx = PreviewContext {
			content: &self.preview.content,
			wrapped_lines: &self.preview.wrapped_lines,
			scroll_offset: self.preview.scroll,
			scrollbar_state: &mut self.preview.scrollbar_state,
			scrollbar_area: &mut self.preview.scrollbar_area,
			scroll_metrics: self.preview.scroll_metrics,
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
