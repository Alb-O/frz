use anyhow::Result;
use frz_core::search_pipeline::SearchOutcome;
use ratatui::crossterm::event::{
	KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::App;

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
				self.results.dragging = false;
			}
			MouseEventKind::Down(MouseButton::Left) if self.results.hovered => {
				if self.select_result_at(mouse.column, mouse.row) && self.preview.enabled {
					self.update_preview();
				}
				self.results.dragging = true;
			}
			MouseEventKind::Up(MouseButton::Left) => {
				self.results.dragging = false;
				self.preview.dragging = false;
			}
			MouseEventKind::Drag(MouseButton::Left) if self.preview.dragging => {
				self.drag_preview_scrollbar_to(mouse.row);
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
		if area.width == 0 || area.height == 0 {
			return false;
		}
		let inside_x = column >= area.x && column < area.x.saturating_add(area.width);
		let inside_y = row >= area.y && row < area.y.saturating_add(area.height);
		inside_x && inside_y
	}

	pub(crate) fn drag_preview_scrollbar_to(&mut self, row: u16) -> bool {
		let Some(area) = self.preview.scrollbar_area else {
			return false;
		};
		if area.height == 0 {
			return false;
		}

		let content_length = self.preview.wrapped_lines.len();
		let max_scroll = self.max_preview_scroll(content_length);

		if max_scroll == 0 {
			self.preview.scroll = 0;
			self.update_scrollbar_state();
			return true;
		}

		let track_start = area.y;
		let track_end = area.y.saturating_add(area.height).saturating_sub(1);
		let clamped_row = row.clamp(track_start, track_end);
		let relative = clamped_row.saturating_sub(track_start) as usize;
		let track_span = area.height.saturating_sub(1) as usize;
		let new_scroll = if track_span == 0 {
			0
		} else {
			max_scroll.saturating_mul(relative.min(track_span)) / track_span
		};

		self.preview.scroll = new_scroll.min(max_scroll);
		self.update_scrollbar_state();
		true
	}
}
