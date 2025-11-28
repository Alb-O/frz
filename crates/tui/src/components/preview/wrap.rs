use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Soft-wrap highlighted lines while preserving a line number gutter and basic indentation.
pub fn wrap_highlighted_lines(
	lines: &[Line<'static>],
	available_width: usize,
) -> Vec<Line<'static>> {
	if available_width == 0 {
		return Vec::new();
	}

	let mut wrapped = Vec::new();

	for line in lines {
		let (gutter, mut body, gutter_width) = split_gutter(line);

		if body.is_empty() {
			wrapped.push(Line::from(gutter.clone()));
			continue;
		}

		// Keep space for the gutter; if it would consume the whole line, fall back to the original.
		if gutter_width >= available_width {
			wrapped.push(line.clone());
			continue;
		}

		let body_width = available_width - gutter_width;
		let continuation_gutter = if gutter_width > 0 {
			Span::raw(" ".repeat(gutter_width))
		} else {
			Span::raw(String::new())
		};
		let continuation_indent = leading_indent_width(&body);

		let mut first = true;
		while !body.is_empty() {
			let (chunk, rest) = take_spans_within_width(&body, body_width);
			body = rest;

			let mut line_spans = if first {
				gutter.clone()
			} else {
				let mut cont = Vec::new();
				if gutter_width > 0 {
					cont.push(continuation_gutter.clone());
				}
				if continuation_indent > 0 {
					cont.push(Span::raw(" ".repeat(continuation_indent)));
				}
				cont
			};

			line_spans.extend(chunk);
			wrapped.push(Line::from(line_spans));
			first = false;
		}
	}

	wrapped
}

fn split_gutter(line: &Line<'static>) -> (Vec<Span<'static>>, Vec<Span<'static>>, usize) {
	let mut gutter = Vec::new();
	let mut body = Vec::new();
	let mut gutter_width = 0;
	let mut found = false;

	for (i, span) in line.spans.iter().cloned().enumerate() {
		if found {
			body.push(span);
			continue;
		}

		let mut before = String::new();
		let mut after = String::new();
		let mut local_width = 0;

		let mut chars = span.content.chars().peekable();
		while let Some(ch) = chars.next() {
			let ch_width = ch.width().unwrap_or(0);
			if ch == '│' {
				found = true;
				before.push(ch);
				local_width += ch_width;

				if chars.peek().copied() == Some(' ') {
					chars.next();
					before.push(' ');
					local_width += 1;
				}

				for ch in chars {
					after.push(ch);
				}
				break;
			} else {
				before.push(ch);
				local_width += ch_width;
			}
		}

		if found {
			if !before.is_empty() {
				gutter.push(Span::styled(before, span.style));
			}
			gutter_width += local_width;

			if !after.is_empty() {
				body.push(Span::styled(after, span.style));
			}

			body.extend(line.spans.iter().skip(i + 1).cloned());
		} else {
			gutter_width += local_width;
			gutter.push(span);
		}
	}

	if !found {
		// No explicit separator; fall back to leading digits/spaces as a gutter.
		let leading_chars = count_leading_gutter_chars(line);
		if leading_chars == 0 {
			gutter.clear();
			gutter_width = 0;
			body = line.spans.clone();
		} else {
			let (gutter_spans, body_spans) = split_spans_at_char(line, leading_chars);
			gutter_width = gutter_spans
				.iter()
				.map(|s| {
					s.content
						.chars()
						.map(|c| c.width().unwrap_or(0))
						.sum::<usize>()
				})
				.sum();
			gutter = gutter_spans;
			body = body_spans;
		}
	}

	(gutter, body, gutter_width)
}

fn count_leading_gutter_chars(line: &Line<'static>) -> usize {
	let mut count = 0;
	for span in &line.spans {
		for ch in span.content.chars() {
			if ch.is_ascii_digit() || ch == ' ' {
				count += 1;
			} else {
				return count;
			}
		}
	}
	count
}

fn split_spans_at_char(
	line: &Line<'static>,
	char_count: usize,
) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
	let mut gutter = Vec::new();
	let mut body = Vec::new();
	let mut consumed = 0;

	for span in &line.spans {
		if consumed >= char_count {
			body.push(span.clone());
			continue;
		}

		let mut prefix = String::new();
		let mut suffix = String::new();
		let mut prefix_bytes = 0;

		for (i, ch) in span.content.char_indices() {
			if consumed < char_count {
				prefix.push(ch);
				consumed += 1;
				prefix_bytes = i + ch.len_utf8();
			} else {
				suffix.push_str(&span.content[i..]);
				break;
			}
		}

		if consumed >= char_count && suffix.is_empty() && prefix_bytes < span.content.len() {
			suffix.push_str(&span.content[prefix_bytes..]);
		}

		if !prefix.is_empty() {
			gutter.push(Span::styled(prefix, span.style));
		}
		if !suffix.is_empty() {
			body.push(Span::styled(suffix, span.style));
		}
	}

	(gutter, body)
}

fn take_spans_within_width(
	spans: &[Span<'static>],
	max_width: usize,
) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
	if max_width == 0 {
		return (Vec::new(), spans.to_vec());
	}

	let mut taken = Vec::new();
	let mut used = 0;
	let mut index = 0;

	while index < spans.len() {
		let span = spans[index].clone();
		let span_text = span.content.to_string();
		let span_width = span_text.width();

		if used + span_width <= max_width {
			used += span_width;
			taken.push(span);
			index += 1;
		} else {
			let remaining_width = max_width.saturating_sub(used);
			if remaining_width == 0 {
				break;
			}

			let (left, right) = split_text_at_width(&span_text, remaining_width);
			if !left.is_empty() {
				taken.push(Span::styled(left, span.style));
			}

			let mut rest = Vec::new();
			if !right.is_empty() {
				rest.push(Span::styled(right, span.style));
			}
			rest.extend_from_slice(&spans[index + 1..]);
			return (taken, rest);
		}
	}

	let rest = spans[index..].to_vec();
	(taken, rest)
}

fn split_text_at_width(text: &str, target_width: usize) -> (String, String) {
	if target_width == 0 {
		return (String::new(), text.to_string());
	}

	let mut width = 0;
	let mut split_byte = 0;

	for (idx, ch) in text.char_indices() {
		let ch_width = ch.width().unwrap_or(0);
		if width + ch_width > target_width {
			break;
		}
		width += ch_width;
		split_byte = idx + ch.len_utf8();
		if width == target_width {
			break;
		}
	}

	let (left, right) = text.split_at(split_byte);
	(left.to_string(), right.to_string())
}

fn leading_indent_width(spans: &[Span<'static>]) -> usize {
	let mut width = 0;

	for span in spans {
		for ch in span.content.chars() {
			match ch {
				' ' => width += 1,
				'\t' => width += 4,
				_ => return width,
			}
		}
	}

	width
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use bat::assets::HighlightingAssets;

	use super::*;

	#[test]
	fn continuations_keep_gutter_padding() {
		let assets = HighlightingAssets::from_binary();
		let content = "    fn long_function_name(arg_one: usize, arg_two: usize, arg_three: usize)";
		let highlighted = crate::components::preview::highlight::highlight_with_bat(
			Path::new("wrap_example.rs"),
			content,
			None,
			16,
			&assets,
		);

		let wrapped = wrap_highlighted_lines(&highlighted, 32);
		assert!(wrapped.len() > 2);
		for (idx, line) in wrapped.iter().enumerate() {
			let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
			let prefix: String = text.chars().take(7).collect();
			if idx == 0 {
				assert!(
					prefix.contains('1'),
					"first line should include line number gutter"
				);
			} else {
				assert_eq!(prefix, "       ", "continuation should keep gutter spacing");
			}
		}
	}
}
