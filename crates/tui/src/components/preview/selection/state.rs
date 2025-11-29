use ratatui::layout::Rect;

/// Represents a text selection within the preview pane.
#[derive(Debug, Clone, Default)]
pub struct TextSelection {
	/// Starting position when drag began (column, row in screen coords).
	pub anchor: Option<(u16, u16)>,
	/// Current drag position (column, row in screen coords).
	pub focus: Option<(u16, u16)>,
	/// Scroll offset when anchor was recorded.
	pub anchor_scroll: usize,
	/// Scroll offset when focus was last updated.
	pub focus_scroll: usize,
	/// Whether we're currently in a drag operation.
	pub selecting: bool,
	/// Whether a selection exists (after mouse up).
	pub active: bool,
}

impl TextSelection {
	/// Create a new empty selection.
	pub fn new() -> Self {
		Self::default()
	}

	/// Start a new selection at the given screen position.
	pub fn start(&mut self, col: u16, row: u16, scroll_offset: usize) {
		self.anchor = Some((col, row));
		self.focus = Some((col, row));
		self.anchor_scroll = scroll_offset;
		self.focus_scroll = scroll_offset;
		self.selecting = true;
		self.active = false;
	}

	/// Update the selection endpoint during drag.
	pub fn update(&mut self, col: u16, row: u16, scroll_offset: usize) {
		if self.selecting {
			self.focus = Some((col, row));
			self.focus_scroll = scroll_offset;
		}
	}

	/// Finish the selection (on mouse up).
	pub fn finish(&mut self) {
		self.selecting = false;
		if let (Some(anchor), Some(focus)) = (self.anchor, self.focus) {
			self.active = anchor != focus;
		}
	}

	/// Clear the selection.
	pub fn clear(&mut self) {
		self.anchor = None;
		self.focus = None;
		self.anchor_scroll = 0;
		self.focus_scroll = 0;
		self.selecting = false;
		self.active = false;
	}

	/// Check if there's an active or in-progress selection.
	pub fn has_selection(&self) -> bool {
		self.selecting || self.active
	}

	/// Get normalized selection bounds (start always before end).
	/// Returns ((start_col, start_row), (end_col, end_row)) in local coords, offset by stored scroll.
	pub fn normalized_bounds(&self, area: Rect) -> Option<((u16, u16), (u16, u16))> {
		let anchor = self.anchor?;
		let focus = self.focus?;

		let anchor_local = (
			anchor.0.saturating_sub(area.x),
			anchor.1.saturating_sub(area.y),
		);
		let focus_local = (
			focus.0.saturating_sub(area.x),
			focus.1.saturating_sub(area.y),
		);

		let anchor_scroll = (self.anchor_scroll as u32).min(u16::MAX as u32) as u16;
		let focus_scroll = (self.focus_scroll as u32).min(u16::MAX as u32) as u16;
		let anchor_local = (anchor_local.0, anchor_local.1.saturating_add(anchor_scroll));
		let focus_local = (focus_local.0, focus_local.1.saturating_add(focus_scroll));

		let (start, end) = if anchor_local.1 < focus_local.1
			|| (anchor_local.1 == focus_local.1 && anchor_local.0 <= focus_local.0)
		{
			(anchor_local, focus_local)
		} else {
			(focus_local, anchor_local)
		};

		Some((start, end))
	}

	/// Check if a given local position is within the selection.
	pub fn contains(&self, col: u16, row: u16, area: Rect) -> bool {
		let Some((start, end)) = self.normalized_bounds(area) else {
			return false;
		};

		if row < start.1 || row > end.1 {
			return false;
		}

		if row == start.1 && row == end.1 {
			col >= start.0 && col < end.0
		} else if row == start.1 {
			col >= start.0
		} else if row == end.1 {
			col < end.0
		} else {
			true
		}
	}
}
