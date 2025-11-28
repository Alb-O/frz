use anyhow::Result;
use frz_core::search_pipeline::SearchOutcome;
use ratatui::crossterm::event::{
	KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;

use super::App;
use crate::components::point_in_rect;

impl<'a> App<'a> {
	/// Process a keyboard event and return a result if the user exits.
	pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Result<Option<SearchOutcome>> {
		match key.code {
			KeyCode::Esc => {
				return Ok(Some(SearchOutcome {
					accepted: false,
					selection: None,
					query: self.search_input.text().to_string(),
				}));
			}
			KeyCode::Enter => {
				let selection = self.current_selection();
				return Ok(Some(SearchOutcome {
					accepted: true,
					selection,
					query: self.search_input.text().to_string(),
				}));
			}
			KeyCode::Tab => {
				self.mark_query_dirty();
				self.switch_mode();
			}
			// Ctrl+P to toggle preview pane
			KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
				self.toggle_preview();
			}
			_ => match key.code {
				KeyCode::Up => {
					self.move_selection_up();
					if self.preview.enabled {
						self.update_preview();
					}
				}
				KeyCode::Down => {
					self.move_selection_down();
					if self.preview.enabled {
						self.update_preview();
					}
				}
				// Ctrl+Up/Down or Shift+Up/Down to scroll preview
				KeyCode::PageUp if self.preview.enabled => {
					self.scroll_preview_up(10);
				}
				KeyCode::PageDown if self.preview.enabled => {
					self.scroll_preview_down(10);
				}
				_ => {
					if self.search_input.input(key) {
						self.mark_query_dirty_from_user_input();
						self.request_search();
					}
				}
			},
		}
		Ok(None)
	}

	pub(crate) fn handle_mouse(&mut self, mouse: MouseEvent) {
		self.update_preview_hover(mouse.column, mouse.row);
		self.update_results_hover(mouse.column, mouse.row);

		match mouse.kind {
			MouseEventKind::ScrollUp if self.preview.enabled && self.preview.hovered => {
				self.scroll_preview_up(3);
			}
			MouseEventKind::ScrollDown if self.preview.enabled && self.preview.hovered => {
				self.scroll_preview_down(3);
			}
			MouseEventKind::ScrollUp if self.results.hovered => {
				self.move_selection_up();
				if self.preview.enabled {
					self.update_preview();
				}
			}
			MouseEventKind::ScrollDown if self.results.hovered => {
				self.move_selection_down();
				if self.preview.enabled {
					self.update_preview();
				}
			}
			MouseEventKind::Down(MouseButton::Left)
				if self.preview.enabled
					&& self.preview_scrollbar_contains(mouse.column, mouse.row) =>
			{
				self.drag_preview_scrollbar_to(mouse.row);
				self.preview.dragging = true;
				self.preview.drag_anchor = None;
				self.results.dragging = false;
			}
			MouseEventKind::Down(MouseButton::Left)
				if self.results_scrollbar_contains(mouse.column, mouse.row) =>
			{
				self.drag_results_scrollbar_to(mouse.row);
				self.results.dragging_scrollbar = true;
				self.results.drag_anchor = None;
				self.results.dragging = false;
				self.preview.dragging = false;
			}
			MouseEventKind::Down(MouseButton::Left) if self.results.hovered => {
				if self.select_result_at(mouse.column, mouse.row) && self.preview.enabled {
					self.update_preview();
				}
				self.results.dragging = true;
			}
			MouseEventKind::Up(MouseButton::Left) => {
				self.results.dragging = false;
				self.results.dragging_scrollbar = false;
				self.results.drag_anchor = None;
				self.preview.drag_anchor = None;
				self.preview.dragging = false;
			}
			MouseEventKind::Drag(MouseButton::Left) if self.preview.dragging => {
				self.drag_preview_scrollbar_to(mouse.row);
			}
			MouseEventKind::Drag(MouseButton::Left) if self.results.dragging_scrollbar => {
				self.drag_results_scrollbar_to(mouse.row);
			}
			MouseEventKind::Drag(MouseButton::Left) if self.results.dragging => {
				if self.select_result_at(mouse.column, mouse.row) && self.preview.enabled {
					self.update_preview();
				}
			}
			_ => {}
		}
	}

	fn switch_mode(&mut self) {
		// No-op now that we only have one mode
	}

	fn move_selection_up(&mut self) {
		if let Some(selected) = self.results.table_state.selected()
			&& selected > 0
		{
			self.results.table_state.select(Some(selected - 1));
		}
	}

	fn move_selection_down(&mut self) {
		if let Some(selected) = self.results.table_state.selected() {
			let len = self.filtered_len();
			if selected + 1 < len {
				self.results.table_state.select(Some(selected + 1));
			}
		}
	}

	fn preview_scrollbar_contains(&self, column: u16, row: u16) -> bool {
		let Some(area) = self.preview.scrollbar_area else {
			return false;
		};
		point_in_rect(column, row, area)
	}

	fn results_scrollbar_contains(&self, column: u16, row: u16) -> bool {
		let Some(area) = self.results.scrollbar_area else {
			return false;
		};
		point_in_rect(column, row, area)
	}

	pub(crate) fn drag_preview_scrollbar_to(&mut self, row: u16) -> bool {
		let Some(area) = self.preview.scrollbar_area else {
			return false;
		};
		if area.height == 0 {
			return false;
		}

		let metrics = self
			.preview
			.scroll_metrics
			.or_else(|| self.preview.compute_scroll_metrics(area.height as usize));
		let Some(metrics) = metrics else {
			return false;
		};

		if !metrics.needs_scrollbar {
			self.preview.scroll = 0;
			self.update_scrollbar_state();
			return true;
		}

		let Some(new_scroll) = drag_with_anchor(
			area,
			row,
			self.preview.scroll,
			metrics.max_scroll,
			metrics.viewport_len,
			metrics.content_length,
			&mut self.preview.drag_anchor,
		) else {
			return false;
		};

		self.preview.scroll = new_scroll.min(metrics.max_scroll);
		self.update_scrollbar_state();
		true
	}

	fn drag_results_scrollbar_to(&mut self, row: u16) -> bool {
		let Some(area) = self.results.scrollbar_area else {
			return false;
		};
		if area.height == 0 {
			return false;
		}

		let content_length = self.results.filtered_len();
		if content_length == 0 {
			return false;
		}

		let metrics = self
			.results
			.scroll_metrics
			.or_else(|| self.results.scroll_metrics(area.height as usize));
		let Some(metrics) = metrics else {
			return false;
		};

		if !metrics.needs_scrollbar {
			*self.results.table_state.offset_mut() = 0;
			if content_length > 0 {
				*self.results.table_state.selected_mut() = Some(
					self.results
						.table_state
						.selected()
						.unwrap_or(0)
						.min(content_length.saturating_sub(1)),
				);
			}
			return true;
		}

		let Some(new_offset) = drag_with_anchor(
			area,
			row,
			self.results.table_state.offset(),
			metrics.max_scroll,
			metrics.viewport_len,
			metrics.content_length,
			&mut self.results.drag_anchor,
		) else {
			return false;
		};

		*self.results.table_state.offset_mut() = new_offset;
		// Keep selection in range without resetting offset.
		let new_selection = new_offset.min(content_length.saturating_sub(1));
		*self.results.table_state.selected_mut() = Some(new_selection);

		// Update preview if enabled
		if self.preview.enabled {
			self.update_preview();
		}

		true
	}
}

fn scrollbar_thumb_height(track_height: usize, viewport_len: usize, content_len: usize) -> usize {
	if track_height == 0 || content_len == 0 {
		return 0;
	}
	let scaled = viewport_len.saturating_mul(track_height);
	let mut thumb = (scaled + content_len.saturating_sub(1)) / content_len;
	if thumb == 0 {
		thumb = 1;
	}
	thumb.min(track_height)
}

fn current_thumb_top(max_scroll: usize, track_span: usize, current_scroll: usize) -> usize {
	if max_scroll == 0 || track_span == 0 {
		0
	} else {
		current_scroll.saturating_mul(track_span) / max_scroll
	}
}

fn drag_with_anchor(
	area: Rect,
	row: u16,
	current: usize,
	max_position: usize,
	viewport_len: usize,
	content_len: usize,
	anchor: &mut Option<u16>,
) -> Option<usize> {
	if area.height == 0 || content_len == 0 {
		return None;
	}

	let track_height = area.height as usize;
	let thumb_height = scrollbar_thumb_height(track_height, viewport_len, content_len);
	let track_span = track_height.saturating_sub(thumb_height);
	if track_span == 0 {
		return Some(0);
	}

	let track_start = area.y;
	let track_end = area.y.saturating_add(area.height).saturating_sub(1);
	let clamped_row = row.clamp(track_start, track_end);
	let relative = clamped_row.saturating_sub(track_start) as usize;

	let thumb_top = current_thumb_top(max_position, track_span, current);
	let anchor_val = match anchor {
		Some(a) => *a as usize,
		None => {
			let a = relative
				.saturating_sub(thumb_top)
				.min(thumb_height.saturating_sub(1));
			*anchor = Some(a as u16);
			a
		}
	};

	let desired_top = relative.saturating_sub(anchor_val).min(track_span);
	Some(max_position.saturating_mul(desired_top) / track_span)
}
