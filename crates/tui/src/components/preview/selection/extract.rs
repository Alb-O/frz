use ratatui::layout::Rect;
use ratatui::text::Line;

use super::gutter::compute_gutter_context;
use super::highlight::selection_bounds_for_line;
use super::state::TextSelection;

/// Extract plain text from selected lines.
pub fn extract_selected_text(
	lines: &[Line<'static>],
	selection: &TextSelection,
	area: Rect,
) -> Option<String> {
	if !selection.has_selection() {
		return None;
	}

	let (start, end) = selection.normalized_bounds(area)?;
	let mut result = String::new();
	let mut prev_gutter_width = 0usize;

	for (line_idx, line) in lines.iter().enumerate() {
		let content_row = line_idx as u16;
		let gutter_ctx = compute_gutter_context(line, prev_gutter_width);
		let is_continuation = gutter_ctx.is_continuation;
		prev_gutter_width = gutter_ctx.next_prev_gutter;

		if content_row < start.1 || content_row > end.1 {
			continue;
		}

		let line_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

		let (sel_start, sel_end) =
			selection_bounds_for_line(content_row, start, end, gutter_ctx.effective_gutter);

		if sel_start >= sel_end {
			continue;
		}

		let chars: Vec<char> = line_text.chars().collect();
		let mut selected: String = chars
			.get(sel_start..sel_end.min(chars.len()))
			.unwrap_or(&[])
			.iter()
			.collect();

		if is_continuation && gutter_ctx.continuation_pad > 0 {
			let mut trimmed = 0usize;
			while trimmed < gutter_ctx.continuation_pad && selected.starts_with(' ') {
				selected.remove(0);
				trimmed += 1;
			}
		}

		if !result.is_empty() && !is_continuation {
			result.push('\n');
		}
		result.push_str(&selected);
	}

	if result.is_empty() {
		None
	} else {
		Some(result)
	}
}
