//! Shared scrollbar rendering component.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::style::Theme;

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

	// Return content area (reduced width to avoid overlap)
	Rect {
		x: area.x,
		y: area.y,
		width: area.width.saturating_sub(1),
		height: area.height,
	}
}
