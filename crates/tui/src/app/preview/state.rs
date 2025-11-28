//! Preview pane state management.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::ScrollbarState;

use crate::components::{PreviewContent, PreviewRuntime};

/// Precomputed scrolling metrics for the preview viewport.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreviewScrollMetrics {
	pub content_length: usize,
	pub viewport_len: usize,
	pub max_scroll: usize,
	pub needs_scrollbar: bool,
}

impl PreviewScrollMetrics {
	/// Compute scroll metrics from content length and viewport height.
	///
	/// Returns default (empty) metrics if either value is zero.
	#[must_use]
	pub fn compute(content_length: usize, viewport_height: usize) -> Self {
		if content_length == 0 || viewport_height == 0 {
			return Self::default();
		}

		let viewport_len = viewport_height.min(content_length).max(1);
		let max_scroll = content_length.saturating_sub(viewport_len);
		let needs_scrollbar = content_length > viewport_len;

		Self {
			content_length,
			viewport_len,
			max_scroll,
			needs_scrollbar,
		}
	}
}

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
	/// Mouse offset into the scrollbar thumb when dragging.
	pub drag_anchor: Option<u16>,
	/// Path of the file whose preview is currently displayed.
	pub path: String,
	/// Path of the file we're currently loading a preview for (if any).
	pub pending_path: Option<String>,
	/// Background preview generation runtime.
	pub runtime: PreviewRuntime,
	/// Cached scroll metrics for the current viewport/content.
	pub scroll_metrics: Option<PreviewScrollMetrics>,
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
			drag_anchor: None,
			path: String::new(),
			pending_path: None,
			runtime: PreviewRuntime::default(),
			scroll_metrics: None,
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

	pub fn compute_scroll_metrics(&self, viewport_height: usize) -> Option<PreviewScrollMetrics> {
		let content_length = self.wrapped_lines.len();
		let metrics = PreviewScrollMetrics::compute(content_length, viewport_height);
		if metrics.content_length == 0 {
			None
		} else {
			Some(metrics)
		}
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
		let Some(metrics) = self.compute_scroll_metrics(self.viewport_height) else {
			self.scrollbar_state = ScrollbarState::default();
			self.scroll_metrics = None;
			return;
		};

		self.scroll_metrics = Some(metrics);
		self.scroll = self.scroll.min(metrics.max_scroll);
		let position =
			self.scrollbar_position(self.scroll, metrics.max_scroll, metrics.content_length);

		self.scrollbar_state = self
			.scrollbar_state
			.content_length(metrics.content_length)
			.viewport_content_length(metrics.viewport_len)
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
