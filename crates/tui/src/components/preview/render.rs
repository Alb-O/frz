//! Preview pane rendering.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, ScrollbarState};

use super::content::{PreviewContent, PreviewKind};
use crate::components::{ScrollMetrics, render_scrollbar};
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
	/// Cached scroll metrics for the current viewport/content.
	pub scroll_metrics: Option<ScrollMetrics>,
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
			let metrics = ctx.scroll_metrics.unwrap_or_else(|| {
				ScrollMetrics::compute(ctx.wrapped_lines.len(), inner.height as usize)
			});

			let visible_lines: Vec<Line<'_>> = ctx
				.wrapped_lines
				.iter()
				.skip(ctx.scroll_offset)
				.take(metrics.viewport_len)
				.cloned()
				.collect();

			let para = Paragraph::new(visible_lines);

			// Render scrollbar only if content overflows
			if metrics.needs_scrollbar {
				let text_area = render_scrollbar(
					frame,
					inner,
					ctx.scrollbar_state,
					ctx.scrollbar_area,
					ctx.theme,
				);
				frame.render_widget(para, text_area);
			} else {
				frame.render_widget(para, inner);
			}
		}
		#[cfg(feature = "media-preview")]
		PreviewKind::Image { image } => {
			image.render(frame, inner);
		}
		#[cfg(feature = "media-preview")]
		PreviewKind::Pdf { pdf } => {
			pdf.image.render(frame, inner);
		}
	}
}
