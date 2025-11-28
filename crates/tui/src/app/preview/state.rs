//! Preview pane state management.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::ScrollbarState;

use crate::components::{PreviewContent, PreviewRuntime};

/// State for the preview pane.
pub(crate) struct PreviewState {
	/// Whether the preview pane is visible.
	pub enabled: bool,
	/// Cached preview content for the currently selected file.
	pub content: PreviewContent,
	/// Scroll offset within the preview pane.
	pub scroll: usize,
	/// Scrollbar state for the preview pane.
	pub scrollbar_state: ScrollbarState,
	/// Last known viewport height for scroll bounds.
	pub viewport_height: usize,
	/// Last known wrap width for the preview content.
	pub wrap_width: usize,
	/// Wrapped preview lines sized to the current viewport width.
	pub wrapped_lines: Vec<Line<'static>>,
	/// Last known preview area on screen.
	pub area: Option<Rect>,
	/// Screen area of the preview scrollbar if rendered.
	pub scrollbar_area: Option<Rect>,
	/// Whether the mouse is currently hovering the preview.
	pub hovered: bool,
	/// Whether the user is dragging the preview scrollbar.
	pub dragging: bool,
	/// Path of the file whose preview is currently displayed.
	pub path: String,
	/// Path of the file we're currently loading a preview for (if any).
	pub pending_path: Option<String>,
	/// Background preview generation runtime.
	pub runtime: PreviewRuntime,
}

impl Default for PreviewState {
	fn default() -> Self {
		Self {
			enabled: false,
			content: PreviewContent::empty(),
			scroll: 0,
			scrollbar_state: ScrollbarState::default(),
			viewport_height: 0,
			wrap_width: 0,
			wrapped_lines: Vec::new(),
			area: None,
			scrollbar_area: None,
			hovered: false,
			dragging: false,
			path: String::new(),
			pending_path: None,
			runtime: PreviewRuntime::default(),
		}
	}
}

impl PreviewState {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn viewport_len(&self, content_length: usize) -> usize {
		if content_length == 0 {
			0
		} else {
			self.viewport_height.max(1).min(content_length)
		}
	}

	pub fn max_scroll(&self, content_length: usize) -> usize {
		let viewport_len = self.viewport_len(content_length);
		content_length.saturating_sub(viewport_len)
	}

	pub fn scroll_up(&mut self, lines: usize) {
		self.scroll = self.scroll.saturating_sub(lines);
		self.update_scrollbar();
	}

	pub fn scroll_down(&mut self, lines: usize) {
		let content_length = self.wrapped_lines.len();
		let max_scroll = self.max_scroll(content_length);
		self.scroll = (self.scroll + lines).min(max_scroll);
		self.update_scrollbar();
	}

	pub fn update_scrollbar(&mut self) {
		let content_length = self.wrapped_lines.len();
		let viewport_len = self.viewport_len(content_length);
		let max_scroll = self.max_scroll(content_length);
		self.scroll = self.scroll.min(max_scroll);
		let position = self.scrollbar_position(self.scroll, max_scroll, content_length);

		self.scrollbar_state = self
			.scrollbar_state
			.content_length(content_length)
			.viewport_content_length(viewport_len)
			.position(position);
	}

	pub fn update_hover(&mut self, column: u16, row: u16) {
		if !self.enabled {
			self.hovered = false;
			return;
		}

		let Some(area) = self.area else {
			self.hovered = false;
			return;
		};

		let inside_x = column >= area.x && column < area.x.saturating_add(area.width);
		let inside_y = row >= area.y && row < area.y.saturating_add(area.height);
		self.hovered = inside_x && inside_y;
	}

	fn scrollbar_position(&self, scroll: usize, max_scroll: usize, content_length: usize) -> usize {
		if max_scroll == 0 || content_length == 0 {
			0
		} else {
			scroll.saturating_mul(content_length.saturating_sub(1)) / max_scroll
		}
	}
}
