use frizbee::Config;
use frz_core::search_pipeline;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::widgets::{HighlightSpacing, Paragraph};
use unicode_width::UnicodeWidthStr;

use super::App;
use super::components::rows::build_file_rows;
use super::components::tables::{
	HIGHLIGHT_SYMBOL, TABLE_COLUMN_SPACING, TABLE_HIGHLIGHT_SPACING, TableSpec,
};
use super::components::{
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

			self.results_area = Some(split[0]);
			self.preview_area = Some(split[1]);
			self.render_results(frame, split[0]);
			self.render_preview_pane(frame, split[1]);
		} else {
			self.preview_area = None;
			self.preview_hovered = false;
			self.results_dragging = false;
			self.results_area = Some(results_area);
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
		let has_selection = self.table_state.selected().is_some();
		let column_widths = resolve_column_widths(area, widths, has_selection);

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

	fn render_preview_pane(&mut self, frame: &mut Frame, area: Rect) {
		// Update viewport height (accounting for borders)
		self.preview_viewport_height = area.height.saturating_sub(2) as usize;
		let inner_width = area.width.saturating_sub(2) as usize;
		let wrap_width = inner_width.saturating_sub(1);
		self.rebuild_preview_wrap(wrap_width);
		self.update_scrollbar_state();

		let ctx = PreviewContext {
			content: &self.preview_content,
			wrapped_lines: &self.preview_wrapped_lines,
			scroll_offset: self.preview_scroll,
			scrollbar_state: &mut self.preview_scrollbar_state,
			scrollbar_area: &mut self.preview_scrollbar_area,
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

pub(crate) fn resolve_column_widths(
	area: Rect,
	widths: &[Constraint],
	has_selection: bool,
) -> Vec<u16> {
	if widths.is_empty() || area.width == 0 {
		return Vec::new();
	}

	let table_width = area.width.saturating_sub(2);
	if table_width == 0 {
		return Vec::new();
	}

	let highlight_width = match TABLE_HIGHLIGHT_SPACING {
		HighlightSpacing::Always => HIGHLIGHT_SYMBOL.width() as u16,
		HighlightSpacing::WhenSelected => {
			if has_selection {
				HIGHLIGHT_SYMBOL.width() as u16
			} else {
				0
			}
		}
		HighlightSpacing::Never => 0,
	};

	let [_selection, columns_area] =
		Layout::horizontal([Constraint::Length(highlight_width), Constraint::Fill(0)])
			.areas(Rect::new(0, 0, table_width, 1));

	Layout::horizontal(widths.to_vec())
		.spacing(TABLE_COLUMN_SPACING)
		.split(columns_area)
		.iter()
		.map(|rect| rect.width)
		.collect()
}

#[cfg(test)]
mod tests {
	use frz_core::search_pipeline::TruncationStyle;

	use super::*;

	#[test]
	fn column_widths_use_table_inner_area() {
		let area = Rect::new(0, 0, 10, 5);
		let widths = [Constraint::Length(20)];
		let resolved = resolve_column_widths(area, &widths, false);

		assert_eq!(resolved, vec![area.width.saturating_sub(2)]);
	}

	#[test]
	fn selection_symbol_reduces_available_width() {
		let area = Rect::new(0, 0, 40, 5);
		let widths = [Constraint::Fill(1), Constraint::Length(8)];

		let without_selection = resolve_column_widths(area, &widths, false);
		let with_selection = resolve_column_widths(area, &widths, true);

		assert_eq!(without_selection.len(), 2);
		assert_eq!(with_selection.len(), 2);
		assert!(with_selection[0] < without_selection[0]);

		let columns_area_without = area.width.saturating_sub(2);
		let total_without: u16 = without_selection.iter().sum::<u16>() + TABLE_COLUMN_SPACING;
		assert!(total_without <= columns_area_without);

		let columns_area_with = area
			.width
			.saturating_sub(2 + HIGHLIGHT_SYMBOL.width() as u16);
		let total_with: u16 = with_selection.iter().sum::<u16>() + TABLE_COLUMN_SPACING;
		assert!(total_with <= columns_area_with);
	}

	#[test]
	fn left_truncated_paths_retain_suffix_with_selection_spacing() {
		let area = Rect::new(0, 0, 50, 5);
		let widths = [Constraint::Fill(1), Constraint::Length(6)];
		let cols = resolve_column_widths(area, &widths, true);
		let path_width = usize::from(cols.first().copied().unwrap_or_default());

		let path = "/very/long/path/to/some/deeply/nested/file_name.ext";
		let truncated =
			crate::highlight::truncate_for_test(path, path_width.max(1), TruncationStyle::Left);

		assert!(truncated.starts_with('…'));
		assert!(truncated.ends_with("file_name.ext"));
	}
}
