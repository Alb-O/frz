//! Preview pane rendering.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
	Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use super::content::{PreviewContent, PreviewKind};
use crate::style::Theme;

/// Context for rendering the preview pane.
pub struct PreviewContext<'a> {
	/// Preview content to render.
	pub content: &'a PreviewContent,
	/// Wrapped lines sized to the current viewport width.
	pub wrapped_lines: &'a [Line<'static>],
	/// Vertical scroll offset (for text content).
	pub scroll_offset: usize,
	/// Scrollbar state for the preview pane.
	pub scrollbar_state: &'a mut ScrollbarState,
	/// Output slot for the rendered scrollbar area (if any).
	pub scrollbar_area: &'a mut Option<Rect>,
	/// Color theme.
	pub theme: &'a Theme,
}

/// Render a centered placeholder message.
fn render_centered_placeholder(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
	let style = Style::default().fg(theme.empty.fg.unwrap_or(ratatui::style::Color::Gray));

	// Vertically center by adding blank lines
	let vertical_padding = area.height.saturating_sub(1) / 2;
	let mut lines: Vec<Line<'_>> = (0..vertical_padding).map(|_| Line::from("")).collect();
	lines.push(Line::from(Span::styled(message, style)));

	let para = Paragraph::new(Text::from(lines)).alignment(Alignment::Center);
	frame.render_widget(para, area);
}

/// Render the preview pane with syntax-highlighted content or image.
pub fn render_preview(frame: &mut Frame, area: Rect, ctx: PreviewContext<'_>) {
	*ctx.scrollbar_area = None;

	let title = if ctx.content.path.is_empty() {
		" Preview ".to_string()
	} else {
		format!(" {} ", ctx.content.path)
	};

	let block = Block::default()
		.borders(Borders::ALL)
		.border_set(ratatui::symbols::border::ROUNDED)
		.border_style(
			Style::default().fg(ctx.theme.header.fg.unwrap_or(ratatui::style::Color::Reset)),
		)
		.title(title);

	let inner = block.inner(area);
	frame.render_widget(block, area);

	match &ctx.content.kind {
		PreviewKind::Placeholder { message } => {
			let msg = if message.is_empty() {
				if ctx.content.path.is_empty() {
					"No file selected"
				} else {
					"Empty file"
				}
			} else {
				message.as_str()
			};
			render_centered_placeholder(frame, inner, msg, ctx.theme);
		}
		PreviewKind::Text { lines: _ } => {
			// Create scrollable view of content
			let visible_lines: Vec<Line<'_>> = ctx
				.wrapped_lines
				.iter()
				.skip(ctx.scroll_offset)
				.take(inner.height as usize)
				.cloned()
				.collect();

			let para = Paragraph::new(visible_lines);

			// Render scrollbar only if content overflows
			let content_length = ctx.wrapped_lines.len();
			let viewport_height = inner.height as usize;

			if content_length > viewport_height {
				let text_area = Rect {
					x: inner.x,
					y: inner.y,
					width: inner.width.saturating_sub(1),
					height: inner.height,
				};

				// Render the text inside the adjusted area so it doesn't overlap the scrollbar
				frame.render_widget(para, text_area);

				let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
					.begin_symbol(None)
					.end_symbol(None)
					.track_symbol(Some("│"))
					.style(
						Style::default().fg(ctx
							.theme
							.header
							.fg
							.unwrap_or(ratatui::style::Color::Reset)),
					);

				// Render scrollbar aligned to the content's right edge
				let scrollbar_area = Rect {
					x: inner.x + inner.width.saturating_sub(1),
					y: inner.y,
					width: 1,
					height: inner.height,
				};
				*ctx.scrollbar_area = Some(scrollbar_area);
				frame.render_stateful_widget(scrollbar, scrollbar_area, ctx.scrollbar_state);
			} else {
				frame.render_widget(para, inner);
			}
		}
		#[cfg(feature = "media-preview")]
		PreviewKind::Image { image } => {
			image.render(frame, inner);
		}
	}
}
