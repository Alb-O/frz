use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::App;
use crate::extensions::api::SearchOutcome;

impl<'a> App<'a> {
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
			_ => match key.code {
				KeyCode::Up => {
					self.move_selection_up();
				}
				KeyCode::Down => {
					self.move_selection_down();
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
