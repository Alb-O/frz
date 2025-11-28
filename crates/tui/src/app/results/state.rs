//! State management for the results table.

use std::collections::HashMap;

use ratatui::layout::Rect;
use ratatui::widgets::TableState;

/// Cache of rendered rows for a specific tab.
#[derive(Default)]
pub(crate) struct TabBuffers {
	pub filtered: Vec<usize>,
	pub scores: Vec<u16>,
	pub headers: Option<Vec<String>>,
	pub widths: Option<Vec<ratatui::layout::Constraint>>,
}

/// Aggregate state for the results table and its interactions.
pub(crate) struct ResultsState {
	/// Selection state for the results table.
	pub table_state: TableState,
	/// Last known results area on screen.
	pub area: Option<Rect>,
	/// Whether the mouse is currently hovering the results table.
	pub hovered: bool,
	/// Whether the user is dragging within the results table.
	pub dragging: bool,
	/// Cache of rendered rows for the active tab.
	pub buffers: TabBuffers,
	/// Stable ID to dataset index mapping.
	pub row_id_map: HashMap<u64, usize>,
}

impl Default for ResultsState {
	fn default() -> Self {
		let mut table_state = TableState::default();
		table_state.select(Some(0));
		Self {
			table_state,
			area: None,
			hovered: false,
			dragging: false,
			buffers: TabBuffers::default(),
			row_id_map: HashMap::new(),
		}
	}
}

impl ResultsState {
	/// Return the number of filtered entries.
	pub fn filtered_len(&self) -> usize {
		self.buffers.filtered.len()
	}

	/// Ensure the row selection remains valid for the currently filtered list.
	pub fn ensure_selection(&mut self) {
		if self.filtered_len() == 0 {
			self.table_state.select(None);
		} else if self.table_state.selected().is_none() {
			self.table_state.select(Some(0));
		} else if let Some(selected) = self.table_state.selected() {
			let len = self.filtered_len();
			if selected >= len {
				self.table_state.select(Some(len.saturating_sub(1)));
			}
		}
	}

	/// Update hover state based on mouse position.
	pub fn update_hover(&mut self, column: u16, row: u16) {
		let Some(area) = self.area else {
			self.hovered = false;
			return;
		};

		let inside_x = column >= area.x && column < area.x.saturating_add(area.width);
		let inside_y = row >= area.y && row < area.y.saturating_add(area.height);
		self.hovered = inside_x && inside_y;
	}

	/// Attempt to select a result at the given mouse position.
	/// Returns true if a selection was made.
	pub fn select_at(&mut self, _column: u16, row: u16) -> bool {
		let Some(area) = self.area else {
			return false;
		};

		// Table is rendered inside a rounded border block; subtract borders.
		let inner_y = area.y.saturating_add(1);
		let inner_width = area.width.saturating_sub(2);
		let inner_height = area.height.saturating_sub(2);
		if inner_width == 0 || inner_height == 0 {
			return false;
		}

		// Header row (1) + bottom margin (1) + separator (1) → rows start at y + 2.
		let body_start_y = inner_y.saturating_add(2);
		if row < body_start_y {
			return false;
		}

		let body_end_y = inner_y.saturating_add(inner_height);
		if row >= body_end_y {
			return false;
		}

		let row_in_view = row.saturating_sub(body_start_y) as usize;
		let visible_index = self.table_state.offset().saturating_add(row_in_view);

		if visible_index >= self.filtered_len() {
			return false;
		}

		self.table_state.select(Some(visible_index));
		true
	}
}
