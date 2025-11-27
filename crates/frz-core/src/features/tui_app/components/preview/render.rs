//! Preview pane rendering.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::content::PreviewContent;
use crate::features::tui_app::style::Theme;

/// Context for rendering the preview pane.
pub struct PreviewContext<'a> {
	/// Preview text content to render.
	pub content: &'a PreviewContent,
	/// Vertical scroll offset.
	pub scroll_offset: usize,
	/// Bat color theme.
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

/// Render the preview pane with syntax-highlighted content.
pub fn render_preview(frame: &mut Frame, area: Rect, ctx: PreviewContext<'_>) {
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

	// Handle placeholder states (errors, loading, no selection)
	if ctx.content.is_placeholder {
		let message = if let Some(error) = &ctx.content.error {
			error.as_str()
		} else if ctx.content.path.is_empty() {
			"No file selected"
		} else {
			"Empty file"
		};
		render_centered_placeholder(frame, inner, message, ctx.theme);
		return;
	}

	// Create scrollable view of content
	let visible_lines: Vec<Line<'_>> = ctx
		.content
		.lines
		.iter()
		.skip(ctx.scroll_offset)
		.take(inner.height as usize)
		.cloned()
		.collect();

	let para = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
	frame.render_widget(para, inner);
}
