use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::gutter::compute_gutter_context;
use super::state::TextSelection;
use crate::style::Theme;

/// Style to apply to selected text.
/// Uses the theme's row_highlight colors for consistency with table selection.
pub fn selection_style(theme: &Theme) -> Style {
	Style::default()
		.bg(theme.row_highlight.bg.unwrap_or(Color::LightBlue))
		.fg(theme.row_highlight.fg.unwrap_or(Color::Black))
		.add_modifier(Modifier::empty())
}

/// Apply selection highlighting to lines for rendering.
/// Takes wrapped lines and returns new lines with selection styling applied.
pub fn apply_selection_to_lines(
	lines: &[Line<'static>],
	selection: &TextSelection,
	area: Rect,
	theme: &Theme,
) -> Vec<Line<'static>> {
	if !selection.has_selection() {
		return lines.to_vec();
	}

	let Some((start, end)) = selection.normalized_bounds(area) else {
		return lines.to_vec();
	};

	let sel_style = selection_style(theme);
	let mut prev_gutter_width = 0usize;

	lines
		.iter()
		.enumerate()
		.map(|(line_idx, line)| {
			let content_row = line_idx as u16;
			let gutter_ctx = compute_gutter_context(line, prev_gutter_width);
			prev_gutter_width = gutter_ctx.next_prev_gutter;

			if content_row < start.1 || content_row > end.1 {
				return line.clone();
			}

			let (sel_start, sel_end) =
				selection_bounds_for_line(content_row, start, end, gutter_ctx.effective_gutter);

			if sel_start >= sel_end {
				return line.clone();
			}

			apply_selection_to_line(line, sel_start, sel_end, sel_style)
		})
		.collect()
}

/// Apply selection styling to a single line.
fn apply_selection_to_line(
	line: &Line<'static>,
	sel_start: usize,
	sel_end: usize,
	sel_style: Style,
) -> Line<'static> {
	let mut new_spans = Vec::new();
	let mut col = 0usize;

	for span in line.spans.iter() {
		let span_len = span.content.chars().count();
		let span_end = col + span_len;

		if span_end <= sel_start || col >= sel_end {
			new_spans.push(span.clone());
		} else if col >= sel_start && span_end <= sel_end {
			new_spans.push(Span::styled(span.content.clone(), sel_style));
		} else {
			let chars: Vec<char> = span.content.chars().collect();
			let mut i = 0;

			if col < sel_start {
				let before_len = sel_start - col;
				let before: String = chars[..before_len].iter().collect();
				new_spans.push(Span::styled(before, span.style));
				i = before_len;
			}

			let sel_local_start = sel_start.saturating_sub(col);
			let sel_local_end = (sel_end - col).min(span_len);
			if sel_local_start < sel_local_end {
				let selected: String = chars[sel_local_start..sel_local_end].iter().collect();
				new_spans.push(Span::styled(selected, sel_style));
				i = sel_local_end;
			}

			if i < span_len {
				let after: String = chars[i..].iter().collect();
				new_spans.push(Span::styled(after, span.style));
			}
		}

		col = span_end;
	}

	Line::from(new_spans)
}

pub(super) fn selection_bounds_for_line(
	content_row: u16,
	start: (u16, u16),
	end: (u16, u16),
	gutter_width: usize,
) -> (usize, usize) {
	let (mut sel_start, mut sel_end) = if content_row == start.1 && content_row == end.1 {
		(start.0 as usize, end.0 as usize)
	} else if content_row == start.1 {
		(start.0 as usize, usize::MAX)
	} else if content_row == end.1 {
		(0, end.0 as usize)
	} else {
		(0, usize::MAX)
	};

	sel_start = sel_start.max(gutter_width);
	if sel_end != usize::MAX {
		sel_end = sel_end.max(sel_start).max(gutter_width);
	}

	if sel_end != usize::MAX && sel_end <= gutter_width {
		return (0, 0);
	}

	if sel_end != usize::MAX && sel_end <= sel_start {
		return (0, 0);
	}

	(sel_start, sel_end)
}
