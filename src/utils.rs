use frizbee::Options;
use frizbee::match_indices;
use ratatui::widgets::{Cell, Row};

use crate::types::{FacetRow, FileRow, TruncationStyle, highlight_cell};

/// Create match indices for the provided needle and configuration.
#[must_use]
pub fn highlight_for_refs(needle: &str, config: Options, text: &str) -> Option<Vec<usize>> {
    if text.is_empty() || needle.is_empty() {
        return None;
    }
    match_indices(needle, text, config).map(|m| m.indices)
}

#[must_use]
pub fn build_facet_rows<'a>(
    filtered_facets: &'a [usize],
    facet_scores: &'a [u16],
    facets: &'a [FacetRow],
    highlight_state: Option<(&'a str, Options)>,
    column_widths: Option<&[u16]>,
) -> Vec<Row<'a>> {
    filtered_facets
        .iter()
        .enumerate()
        .filter_map(|(idx, &actual_index)| {
            let facet = facets.get(actual_index)?;
            let score = facet_scores.get(idx).copied().unwrap_or_default();
            let highlight = highlight_state
                .and_then(|(needle, config)| highlight_for_refs(needle, config, &facet.name));
            let name_width = column_widths.and_then(|widths| widths.first()).copied();
            Some(Row::new([
                highlight_cell(&facet.name, highlight, name_width, TruncationStyle::Right),
                Cell::from(facet.count.to_string()),
                Cell::from(score.to_string()),
            ]))
        })
        .collect()
}

#[must_use]
pub fn build_file_rows<'a>(
    filtered_files: &'a [usize],
    file_scores: &'a [u16],
    files: &'a [FileRow],
    highlight_state: Option<(&'a str, Options)>,
    column_widths: Option<&[u16]>,
) -> Vec<Row<'a>> {
    filtered_files
        .iter()
        .enumerate()
        .filter_map(|(idx, &actual_index)| {
            let entry = files.get(actual_index)?;
            let score = file_scores.get(idx).copied().unwrap_or_default();
            let path_highlight = highlight_state
                .and_then(|(needle, config)| highlight_for_refs(needle, config, &entry.path));
            let tag_highlight = highlight_state.and_then(|(needle, config)| {
                highlight_for_refs(needle, config, &entry.display_tags)
            });
            let (path_width, tag_width) = column_widths
                .map(|widths| {
                    let path = widths.first().copied();
                    let tags = widths.get(1).copied();
                    (path, tags)
                })
                .unwrap_or((None, None));
            Some(Row::new([
                highlight_cell(
                    &entry.path,
                    path_highlight,
                    path_width,
                    entry.truncation_style(),
                ),
                highlight_cell(
                    &entry.display_tags,
                    tag_highlight,
                    tag_width,
                    TruncationStyle::Right,
                ),
                Cell::from(score.to_string()),
            ]))
        })
        .collect()
}
