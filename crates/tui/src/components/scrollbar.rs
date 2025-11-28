//! Shared scrollbar rendering component.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::style::Theme;

/// Precomputed scrolling metrics for a scrollable viewport.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollMetrics {
	/// Total number of items in the content.
	pub content_length: usize,
	/// Number of items visible in the viewport.
	pub viewport_len: usize,
	/// Maximum scroll/offset position.
	pub max_scroll: usize,
	/// Whether content overflows and needs a scrollbar.
	pub needs_scrollbar: bool,
}

impl ScrollMetrics {
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

	/// Convert scroll position to scrollbar position for rendering.
	#[must_use]
	pub fn scrollbar_position(&self, scroll: usize) -> usize {
		if self.max_scroll == 0 || self.content_length == 0 {
			0
		} else {
			scroll.saturating_mul(self.content_length.saturating_sub(1)) / self.max_scroll
		}
	}
}

/// Check if a point (column, row) is inside a rectangle.
#[must_use]
pub fn point_in_rect(column: u16, row: u16, area: Rect) -> bool {
	if area.width == 0 || area.height == 0 {
		return false;
	}
	let inside_x = column >= area.x && column < area.x.saturating_add(area.width);
	let inside_y = row >= area.y && row < area.y.saturating_add(area.height);
	inside_x && inside_y
}

/// Render a themed vertical scrollbar on the right side of the given area.
///
/// # Arguments
/// * `frame` - The frame to render into
/// * `area` - The full area (scrollbar will be placed on the right edge)
/// * `scrollbar_state` - The scrollbar state to render
/// * `scrollbar_area` - Output parameter to store the scrollbar's rendered area
/// * `theme` - The theme to use for styling
///
/// # Returns
/// The area that should be used for content (with width reduced by 1 if scrollbar is rendered).
pub fn render_scrollbar(
	frame: &mut Frame,
	area: Rect,
	scrollbar_state: &mut ScrollbarState,
	scrollbar_area: &mut Option<Rect>,
	theme: &Theme,
) -> Rect {
	let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
		.begin_symbol(None)
		.end_symbol(None)
		.track_symbol(Some("│"))
		.style(Style::default().fg(theme.header.fg.unwrap_or(ratatui::style::Color::Reset)));

	let sb_area = Rect {
		x: area.x + area.width.saturating_sub(1),
		y: area.y,
		width: 1,
		height: area.height,
	};

	*scrollbar_area = Some(sb_area);
	frame.render_stateful_widget(scrollbar, sb_area, scrollbar_state);

	Rect {
		x: area.x,
		y: area.y,
		width: area.width.saturating_sub(1),
		height: area.height,
	}
}
