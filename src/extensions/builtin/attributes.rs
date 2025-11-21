use std::sync::atomic::AtomicU64;

use ratatui::layout::{Constraint, Layout, Rect};

use crate::extensions::api::{
	SearchData, SearchMode, SearchSelection, SearchStream, stream_attributes,
};
use crate::tui::components::tables::{TableRenderContext, TableSpec};
use crate::tui::tables::rows::build_facet_rows;
use crate::tui::theme::Theme;

pub const DATASET_KEY: &str = "attributes";

pub const fn mode() -> SearchMode {
	SearchMode::Attributes
}

pub fn default_headers() -> Vec<String> {
	vec!["Tag".into(), "Count".into(), "Score".into()]
}

pub fn default_widths() -> Vec<Constraint> {
	vec![
		Constraint::Percentage(50),
		Constraint::Length(8),
		Constraint::Length(8),
	]
}

pub fn table<'a>(context: TableRenderContext<'a>, theme: &'a Theme) -> TableSpec<'a> {
	let widths = context.widths.cloned().unwrap_or_else(default_widths);
	let headers = context.headers.cloned().unwrap_or_else(default_headers);
	let column_widths = resolve_column_widths(context.area, &widths);
	let rows = build_facet_rows(
		context.filtered,
		context.scores,
		&context.data.attributes,
		context.highlight,
		theme.highlight_style(),
		Some(&column_widths),
	);

	TableSpec {
		headers,
		widths,
		rows,
	}
}

pub fn selection(data: &SearchData, index: usize) -> Option<SearchSelection> {
	data.attributes
		.get(index)
		.cloned()
		.map(SearchSelection::Attribute)
}

pub fn stream(
	data: &SearchData,
	query: &str,
	stream: SearchStream<'_>,
	latest_query_id: &AtomicU64,
) -> bool {
	stream_attributes(data, query, stream, latest_query_id)
}

fn resolve_column_widths(area: Rect, widths: &[Constraint]) -> Vec<u16> {
	if widths.is_empty() || area.width == 0 {
		return Vec::new();
	}

	let layout_area = Rect {
		x: 0,
		y: 0,
		width: area.width,
		height: 1,
	};
	Layout::horizontal(widths.to_vec())
		.spacing(1)
		.split(layout_area)
		.iter()
		.map(|rect| rect.width)
		.collect()
}
