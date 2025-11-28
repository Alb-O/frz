use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::HighlightSpacing;
use unicode_width::UnicodeWidthStr;

use crate::components::tables::{HIGHLIGHT_SYMBOL, TABLE_COLUMN_SPACING, TABLE_HIGHLIGHT_SPACING};

pub(crate) fn resolve_column_widths(
	area: Rect,
	widths: &[Constraint],
	has_selection: bool,
) -> Vec<u16> {
	if widths.is_empty() || area.width == 0 {
		return Vec::new();
	}

	let table_width = area.width.saturating_sub(2);
	if table_width == 0 {
		return Vec::new();
	}

	let highlight_width = match TABLE_HIGHLIGHT_SPACING {
		HighlightSpacing::Always => HIGHLIGHT_SYMBOL.width() as u16,
		HighlightSpacing::WhenSelected => {
			if has_selection {
				HIGHLIGHT_SYMBOL.width() as u16
			} else {
				0
			}
		}
		HighlightSpacing::Never => 0,
	};

	let [_selection, columns_area] =
		Layout::horizontal([Constraint::Length(highlight_width), Constraint::Fill(0)])
			.areas(Rect::new(0, 0, table_width, 1));

	Layout::horizontal(widths.to_vec())
		.spacing(TABLE_COLUMN_SPACING)
		.split(columns_area)
		.iter()
		.map(|rect| rect.width)
		.collect()
}

#[cfg(test)]
mod tests {
	use frz_core::search_pipeline::TruncationStyle;

	use super::*;

	#[test]
	fn column_widths_use_table_inner_area() {
		let area = Rect::new(0, 0, 10, 5);
		let widths = [Constraint::Length(20)];
		let resolved = resolve_column_widths(area, &widths, false);

		assert_eq!(resolved, vec![area.width.saturating_sub(2)]);
	}

	#[test]
	fn selection_symbol_reduces_available_width() {
		let area = Rect::new(0, 0, 40, 5);
		let widths = [Constraint::Fill(1), Constraint::Length(8)];

		let without_selection = resolve_column_widths(area, &widths, false);
		let with_selection = resolve_column_widths(area, &widths, true);

		assert_eq!(without_selection.len(), 2);
		assert_eq!(with_selection.len(), 2);
		assert!(with_selection[0] < without_selection[0]);

		let columns_area_without = area.width.saturating_sub(2);
		let total_without: u16 = without_selection.iter().sum::<u16>() + TABLE_COLUMN_SPACING;
		assert!(total_without <= columns_area_without);

		let columns_area_with = area
			.width
			.saturating_sub(2 + HIGHLIGHT_SYMBOL.width() as u16);
		let total_with: u16 = with_selection.iter().sum::<u16>() + TABLE_COLUMN_SPACING;
		assert!(total_with <= columns_area_with);
	}

	#[test]
	fn left_truncated_paths_retain_suffix_with_selection_spacing() {
		let area = Rect::new(0, 0, 50, 5);
		let widths = [Constraint::Fill(1), Constraint::Length(6)];
		let cols = resolve_column_widths(area, &widths, true);
		let path_width = usize::from(cols.first().copied().unwrap_or_default());

		let path = "/very/long/path/to/some/deeply/nested/file_name.ext";
		let truncated =
			crate::highlight::truncate_for_test(path, path_width.max(1), TruncationStyle::Left);

		assert!(truncated.starts_with('…'));
		assert!(truncated.ends_with("file_name.ext"));
	}
}
