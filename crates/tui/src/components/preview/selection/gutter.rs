use ratatui::text::Line;
use unicode_width::UnicodeWidthChar;

/// Captures gutter-related offsets for a wrapped line.
#[derive(Debug, Clone, Copy)]
pub struct GutterContext {
	/// Total columns to skip before selectable text (including wrap pad).
	pub effective_gutter: usize,
	/// Extra padding introduced by a wrapped continuation.
	pub continuation_pad: usize,
	/// Whether this line is a wrapped continuation of the previous line.
	pub is_continuation: bool,
	/// Gutter width to carry forward for subsequent lines.
	pub next_prev_gutter: usize,
}

/// Compute gutter information for the given line, preserving line-number gutter and
/// wrap padding so selection avoids false indentation.
pub fn compute_gutter_context(line: &Line<'static>, prev_gutter_width: usize) -> GutterContext {
	let gutter_width = gutter_width(line);
	let leading_spaces = leading_space_width(line);
	let is_continuation = gutter_width == 0 && prev_gutter_width > 0;

	let base_gutter = if is_continuation {
		prev_gutter_width.min(leading_spaces)
	} else {
		gutter_width
	};

	let continuation_pad = if base_gutter == 0 {
		0
	} else {
		leading_spaces.saturating_sub(base_gutter)
	};
	let effective_gutter = base_gutter.saturating_add(continuation_pad);

	let next_prev_gutter = if gutter_width > 0 {
		gutter_width
	} else {
		prev_gutter_width
	};

	GutterContext {
		effective_gutter,
		continuation_pad,
		is_continuation,
		next_prev_gutter,
	}
}

fn gutter_width(line: &Line<'static>) -> usize {
	let mut width = 0usize;
	let mut saw_digit = false;
	let mut took_separator_space = false;

	for span in &line.spans {
		let mut chars = span.content.chars().peekable();
		while let Some(ch) = chars.next() {
			let ch_width = ch.width().unwrap_or(0);
			if ch == '│' {
				width = width.saturating_add(ch_width);
				if chars.peek() == Some(&' ') {
					chars.next();
					width = width.saturating_add(1);
				}
				return width.max(1);
			}

			if ch.is_ascii_digit() {
				width = width.saturating_add(ch_width);
				saw_digit = true;
				continue;
			}

			if ch == ' ' {
				if saw_digit && !took_separator_space {
					width = width.saturating_add(ch_width);
					took_separator_space = true;
					continue;
				}
				if !saw_digit {
					width = width.saturating_add(ch_width);
					continue;
				}
				return width;
			}

			return if saw_digit { width } else { 0 };
		}
	}

	if saw_digit { width } else { 0 }
}

fn leading_space_width(line: &Line<'static>) -> usize {
	let mut width = 0usize;
	for span in &line.spans {
		for ch in span.content.chars() {
			if ch == ' ' || ch == '\t' {
				width = width.saturating_add(ch.width().unwrap_or(0));
			} else {
				return width;
			}
		}
	}
	width
}
