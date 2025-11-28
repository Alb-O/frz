//! Preview pane rendering.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
	Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use super::content::{PreviewContent, PreviewKind};
use crate::style::Theme;

/// Context for rendering the preview pane.
pub struct PreviewContext<'a> {
	/// Preview content to render.
	pub content: &'a PreviewContent,
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
	let style = Style::default().fg(theme
		.empty_style()
		.fg
		.unwrap_or(ratatui::style::Color::Gray));

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
		.border_style(Style::default().fg(ctx.theme.header_fg()))
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
		PreviewKind::Text { lines } => {
			// Create scrollable view of content
			let visible_lines: Vec<Line<'_>> = lines
				.iter()
				.skip(ctx.scroll_offset)
				.take(inner.height as usize)
				.cloned()
				.collect();

			let para = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
			frame.render_widget(para, inner);

			// Render scrollbar only if content overflows
			let content_length = lines.len();
			let viewport_height = inner.height as usize;

			if content_length > viewport_height {
				let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
					.begin_symbol(None)
					.end_symbol(None)
					.track_symbol(Some("│"));

				// Render scrollbar aligned to the content's right edge
				let scrollbar_area = Rect {
					x: inner.x + inner.width.saturating_sub(1),
					y: inner.y,
					width: 1,
					height: inner.height,
				};
				*ctx.scrollbar_area = Some(scrollbar_area);
				frame.render_stateful_widget(scrollbar, scrollbar_area, ctx.scrollbar_state);
			}
		}
		#[cfg(feature = "media-preview")]
		PreviewKind::Image { image } => {
			image.render(frame, inner);
		}
	}
}
