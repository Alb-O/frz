use frizbee::Options;
use frizbee::match_indices;
use ratatui::widgets::{Cell, Row};

use crate::types::{FacetRow, FileRow, highlight_cell};

/// Create match indices for the provided needle and configuration.
pub fn highlight_for_refs(needle: &str, config: Options, text: &str) -> Option<Vec<usize>> {
    if text.is_empty() || needle.is_empty() {
        return None;
    }
    match_indices(needle, text, config).map(|m| m.indices)
}

pub fn build_facet_rows<'a>(
    filtered_facets: &'a [usize],
    facet_scores: &'a [u16],
    facets: &'a [FacetRow],
    highlight_state: Option<(&'a str, Options)>,
) -> Vec<Row<'a>> {
    filtered_facets
        .iter()
        .enumerate()
        .map(|(idx, &actual_index)| {
            let facet = &facets[actual_index];
            let score = facet_scores.get(idx).copied().unwrap_or_default();
            let highlight = highlight_state
                .and_then(|(needle, config)| highlight_for_refs(needle, config, &facet.name));
            Row::new([
                highlight_cell(&facet.name, highlight),
                Cell::from(facet.count.to_string()),
                Cell::from(score.to_string()),
            ])
        })
        .collect()
}

pub fn build_file_rows<'a>(
    filtered_files: &'a [usize],
    file_scores: &'a [u16],
    files: &'a [FileRow],
    highlight_state: Option<(&'a str, Options)>,
) -> Vec<Row<'a>> {
    filtered_files
        .iter()
        .enumerate()
        .map(|(idx, &actual_index)| {
            let entry = &files[actual_index];
            let score = file_scores.get(idx).copied().unwrap_or_default();
            let path_highlight = highlight_state
                .and_then(|(needle, config)| highlight_for_refs(needle, config, &entry.path));
            let tag_highlight = highlight_state.and_then(|(needle, config)| {
                highlight_for_refs(needle, config, &entry.display_tags)
            });
            Row::new([
                highlight_cell(&entry.path, path_highlight),
                highlight_cell(&entry.display_tags, tag_highlight),
                Cell::from(score.to_string()),
            ])
        })
        .collect()
}
