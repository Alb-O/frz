use anyhow::Result;
use frz_core::search_pipeline::SearchOutcome;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

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
					if self.preview_enabled {
						self.update_preview();
					}
				}
				KeyCode::Down => {
					self.move_selection_down();
					if self.preview_enabled {
						self.update_preview();
					}
				}
				// Ctrl+Up/Down or Shift+Up/Down to scroll preview
				KeyCode::PageUp if self.preview_enabled => {
					self.scroll_preview_up(10);
				}
				KeyCode::PageDown if self.preview_enabled => {
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

		if self.preview_enabled && self.preview_hovered {
			match mouse.kind {
				MouseEventKind::ScrollUp => self.scroll_preview_up(3),
				MouseEventKind::ScrollDown => self.scroll_preview_down(3),
				_ => {}
			}
		} else if self.results_hovered {
			match mouse.kind {
				MouseEventKind::ScrollUp => {
					self.move_selection_up();
					if self.preview_enabled {
						self.update_preview();
					}
				}
				MouseEventKind::ScrollDown => {
					self.move_selection_down();
					if self.preview_enabled {
						self.update_preview();
					}
				}
				_ => {}
			}
		}
	}

	fn switch_mode(&mut self) {
		// No-op now that we only have one mode
	}

	fn move_selection_up(&mut self) {
		if let Some(selected) = self.table_state.selected()
			&& selected > 0
		{
			self.table_state.select(Some(selected - 1));
		}
	}

	fn move_selection_down(&mut self) {
		if let Some(selected) = self.table_state.selected() {
			let len = self.filtered_len();
			if selected + 1 < len {
				self.table_state.select(Some(selected + 1));
			}
		}
	}
}
