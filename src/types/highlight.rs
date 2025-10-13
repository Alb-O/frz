use std::mem;

use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Cell;
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

use super::TruncationStyle;

/// Build a table cell that highlights matching indices within `text`.
pub(crate) fn highlight_cell(
    text: &str,
    indices: Option<Vec<usize>>,
    max_width: Option<u16>,
    truncation: TruncationStyle,
) -> Cell<'_> {
    let (display_text, indices) = if let Some(width) = max_width.map(usize::from) {
        truncate_with_highlight(text, indices, width, truncation)
    } else {
        (text.to_string(), indices)
    };

    let Some(mut sorted_indices) = indices.filter(|indices| !indices.is_empty()) else {
        return Cell::from(display_text);
    };
    sorted_indices.sort_unstable();
    let mut next = sorted_indices.into_iter().peekable();
    let mut buffer = String::new();
    let mut highlighted = false;
    let mut spans = Vec::new();
    let theme = Theme::default();
    let highlight_style = theme.highlight_style();

    for (idx, ch) in display_text.chars().enumerate() {
        let should_highlight = next.peek().copied() == Some(idx);
        if should_highlight {
            next.next();
        }
        if should_highlight != highlighted {
            if !buffer.is_empty() {
                let style = if highlighted {
                    highlight_style
                } else {
                    Style::default()
                };
                spans.push(Span::styled(mem::take(&mut buffer), style));
            }
            highlighted = should_highlight;
        }
        buffer.push(ch);
    }

    if !buffer.is_empty() {
        let style = if highlighted {
            highlight_style
        } else {
            Style::default()
        };
        spans.push(Span::styled(buffer, style));
    }

    Cell::from(Text::from(Line::from(spans)))
}

fn truncate_with_highlight(
    text: &str,
    indices: Option<Vec<usize>>,
    max_width: usize,
    truncation: TruncationStyle,
) -> (String, Option<Vec<usize>>) {
    if max_width == 0 {
        return (String::new(), None);
    }

    let original_width = text.width();
    if original_width <= max_width {
        return (text.to_string(), indices);
    }

    let ellipsis = "…";
    let ellipsis_width = ellipsis.width();
    if max_width <= ellipsis_width {
        return (ellipsis.to_string(), None);
    }

    let available = max_width - ellipsis_width;
    match truncation {
        TruncationStyle::Right => {
            let (slice, _) = text.unicode_truncate(available);
            let mut truncated = slice.to_string();
            truncated.push_str(ellipsis);
            let limit = slice.chars().count();
            let indices = indices.and_then(|indices| {
                let adjusted: Vec<usize> = indices.into_iter().filter(|&idx| idx < limit).collect();
                (!adjusted.is_empty()).then_some(adjusted)
            });
            (truncated, indices)
        }
        TruncationStyle::Left => {
            let (slice, _) = text.unicode_truncate_start(available);
            let mut truncated = ellipsis.to_string();
            truncated.push_str(slice);
            let slice_len = slice.chars().count();
            let total_chars = text.chars().count();
            let trimmed = total_chars.saturating_sub(slice_len);
            let indices = indices.and_then(|indices| {
                let adjusted: Vec<usize> = indices
                    .into_iter()
                    .filter_map(|idx| idx.checked_sub(trimmed))
                    .filter(|&idx| idx < slice_len)
                    .map(|idx| idx + 1)
                    .collect();
                (!adjusted.is_empty()).then_some(adjusted)
            });
            (truncated, indices)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_truncation_keeps_indices() {
        let (text, indices) =
            truncate_with_highlight("abcdefgh", Some(vec![1, 3, 6]), 5, TruncationStyle::Right);
        assert_eq!(text, "abcd…");
        assert_eq!(indices, Some(vec![1, 3]));
    }

    #[test]
    fn left_truncation_adjusts_indices() {
        let (text, indices) =
            truncate_with_highlight("abcdefgh", Some(vec![1, 3, 6]), 5, TruncationStyle::Left);
        assert_eq!(text, "…efgh");
        assert_eq!(indices, Some(vec![3]));
    }
}
