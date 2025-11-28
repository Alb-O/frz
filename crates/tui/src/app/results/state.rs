//! State management for the results table.

use std::collections::HashMap;

use ratatui::layout::Rect;
use ratatui::widgets::{ScrollbarState, TableState};

use crate::components::tables::TABLE_HEADER_ROWS;
use crate::components::{ScrollMetrics, point_in_rect};

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
	/// Scrollbar state for the results table.
	pub scrollbar_state: ScrollbarState,
	/// Screen area of the scrollbar if rendered.
	pub scrollbar_area: Option<Rect>,
	/// Last known results area on screen.
	pub area: Option<Rect>,
	/// Whether the mouse is currently hovering the results table.
	pub hovered: bool,
	/// Whether the user is dragging within the results table.
	pub dragging: bool,
	/// Whether the user is dragging the results scrollbar.
	pub dragging_scrollbar: bool,
	/// Mouse offset into the scrollbar thumb when dragging.
	pub drag_anchor: Option<u16>,
	/// Cache of rendered rows for the active tab.
	pub buffers: TabBuffers,
	/// Stable ID to dataset index mapping.
	pub row_id_map: HashMap<u64, usize>,
	/// Cached scroll metrics based on the last rendered viewport.
	pub scroll_metrics: Option<ScrollMetrics>,
}

impl Default for ResultsState {
	fn default() -> Self {
		let mut table_state = TableState::default();
		table_state.select(Some(0));
		Self {
			table_state,
			scrollbar_state: ScrollbarState::default(),
			scrollbar_area: None,
			area: None,
			hovered: false,
			dragging: false,
			dragging_scrollbar: false,
			drag_anchor: None,
			buffers: TabBuffers::default(),
			row_id_map: HashMap::new(),
			scroll_metrics: None,
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

		self.hovered = point_in_rect(column, row, area);
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

	/// Compute scroll/offset metrics for the results viewport.
	///
	/// Uses `ScrollMetrics` but accounts for table header rows.
	pub fn scroll_metrics(&self, viewport_height: usize) -> Option<ScrollMetrics> {
		let content_length = self.filtered_len();
		if content_length == 0 || viewport_height == 0 {
			return None;
		}

		let available_rows = viewport_height.saturating_sub(TABLE_HEADER_ROWS);
		// Use the unified ScrollMetrics::compute for the available rows
		let metrics = ScrollMetrics::compute(content_length, available_rows);
		if metrics.content_length == 0 {
			None
		} else {
			Some(metrics)
		}
	}

	/// Update scrollbar state to match current table content and scroll position.
	pub fn update_scrollbar(&mut self, viewport_height: usize) {
		let Some(metrics) = self.scroll_metrics(viewport_height) else {
			self.scrollbar_state = ScrollbarState::default();
			self.scroll_metrics = None;
			return;
		};

		self.scroll_metrics = Some(metrics);
		if !metrics.needs_scrollbar {
			*self.table_state.offset_mut() = 0;
			if let Some(selected) = self.table_state.selected() {
				self.table_state
					.select(Some(selected.min(metrics.content_length.saturating_sub(1))));
			}
			self.scrollbar_state = ScrollbarState::default();
			return;
		}

		let offset = self.table_state.offset().min(metrics.max_scroll);
		*self.table_state.offset_mut() = offset;
		if let Some(selected) = self.table_state.selected() {
			self.table_state
				.select(Some(selected.min(metrics.content_length.saturating_sub(1))));
		}

		let position = metrics.scrollbar_position(offset);

		self.scrollbar_state = self
			.scrollbar_state
			.content_length(metrics.content_length)
			.viewport_content_length(metrics.viewport_len)
			.position(position);
	}
}
