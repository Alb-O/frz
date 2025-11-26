//! Preview pane rendering.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::content::PreviewContent;
use crate::ui::style::Theme;

/// Context for rendering the preview pane.
pub struct PreviewContext<'a> {
	pub content: &'a PreviewContent,
	pub scroll_offset: usize,
	pub theme: &'a Theme,
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
		.border_style(Style::default().fg(ctx.theme.header_fg()))
		.title(title);

	let inner = block.inner(area);
	frame.render_widget(block, area);

	if let Some(error) = &ctx.content.error {
		let error_text = Text::from(vec![
			Line::from(""),
			Line::from(Span::styled(
				error.clone(),
				Style::default().fg(ctx
					.theme
					.empty_style()
					.fg
					.unwrap_or(ratatui::style::Color::Gray)),
			)),
		]);
		let para = Paragraph::new(error_text);
		frame.render_widget(para, inner);
		return;
	}

	if ctx.content.lines.is_empty() {
		let empty_text = Text::from(vec![
			Line::from(""),
			Line::from(Span::styled("No file selected", ctx.theme.empty_style())),
		]);
		let para = Paragraph::new(empty_text);
		frame.render_widget(para, inner);
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
