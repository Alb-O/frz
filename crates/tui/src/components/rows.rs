use frizbee::{Config, match_indices};
use frz_core::filesystem::search::FileRow;
use ratatui::style::Style;
use ratatui::widgets::{Cell, Row};

use crate::highlight::highlight_cell_with_prefix;

/// Create match indices for the provided needle and configuration.
#[must_use]
pub fn highlight_for_refs(needle: &str, config: &Config, text: &str) -> Option<Vec<usize>> {
	if text.is_empty() || needle.is_empty() {
		return None;
	}
	match_indices(needle, text, config).map(|m| m.indices)
}

/// Build table rows for the filtered file results.
#[must_use]
pub fn build_file_rows<'a>(
	filtered_files: &'a [usize],
	file_scores: &'a [u16],
	files: &'a [FileRow],
	highlight_state: Option<(&'a str, Config)>,
	highlight_style: Style,
	column_widths: Option<&[u16]>,
) -> Vec<Row<'a>> {
	filtered_files
		.iter()
		.enumerate()
		.filter_map(|(idx, &actual_index)| {
			let entry = files.get(actual_index)?;
			let score = file_scores.get(idx).copied().unwrap_or_default();
			let path_highlight = highlight_state
				.as_ref()
				.and_then(|(needle, config)| highlight_for_refs(needle, config, &entry.path));
			// Leave one column of slack so we don't rely on the table drawing right up to the edge.
			let path_width = column_widths
				.and_then(|widths| widths.first().copied())
				.map(|w| w.saturating_sub(1));
			Some(Row::new([
				highlight_cell_with_prefix(
					&entry.path,
					path_highlight,
					path_width,
					entry.truncation_style(),
					highlight_style,
					None,
				),
				Cell::from(score.to_string()),
			]))
		})
		.collect()
}
