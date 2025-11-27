use frizbee::Config;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table};

use crate::style::Theme;
use frz_core::features::search_pipeline::SearchData;

const HIGHLIGHT_SYMBOL: &str = "▶ ";
const TABLE_COLUMN_SPACING: u16 = 1;

/// Fully materialized table configuration.
pub struct TableSpec<'a> {
	/// Column headers.
	pub headers: Vec<String>,
	/// Column width constraints.
	pub widths: Vec<Constraint>,
	/// Rendered table rows.
	pub rows: Vec<Row<'a>>,
	/// Optional title for the bordered table.
	pub title: Option<String>,
}

/// Argument bundle describing the data a table render should use.
pub struct TableRenderContext<'a> {
	/// Rendering area.
	pub area: Rect,
	/// Indices of filtered rows to display.
	pub filtered: &'a [usize],
	/// Relevance scores for each row.
	pub scores: &'a [u16],
	/// Optional custom column headers.
	pub headers: Option<&'a Vec<String>>,
	/// Optional column width constraints.
	pub widths: Option<&'a Vec<Constraint>>,
	/// Optional highlight configuration for search terms.
	pub highlight: Option<(&'a str, Config)>,
	/// Data source for the table.
	pub data: &'a SearchData,
}

/// Render the table using the provided dataset definition.
pub fn render_table(
	frame: &mut Frame,
	area: Rect,
	table_state: &mut ratatui::widgets::TableState,
	spec: TableSpec<'_>,
	theme: &Theme,
) {
	let mut block = Block::default()
		.borders(Borders::ALL)
		.border_set(ratatui::symbols::border::ROUNDED)
		.border_style(Style::default().fg(theme.header_fg()));

	if let Some(title) = spec.title.clone() {
		block = block.title(title);
	}

	let inner = block.inner(area);
	frame.render_widget(block, area);

	render_configured_table(
		frame,
		inner,
		table_state,
		HighlightSpacing::WhenSelected,
		theme,
		spec,
	);
}

fn render_configured_table(
	frame: &mut Frame,
	area: Rect,
	table_state: &mut ratatui::widgets::TableState,
	highlight_spacing: HighlightSpacing,
	theme: &Theme,
	spec: TableSpec<'_>,
) {
	let header_cells = spec.headers.into_iter().map(Cell::from).collect::<Vec<_>>();
	let header_style = Style::default().fg(theme.header_fg());
	let header = Row::new(header_cells)
		.style(header_style)
		.height(1)
		.bottom_margin(1);

	let mut widths = spec.widths;
	if widths.is_empty() {
		widths = vec![Constraint::Fill(1)];
	}

	let table = Table::new(spec.rows, widths)
		.header(header)
		.column_spacing(TABLE_COLUMN_SPACING)
		.highlight_spacing(highlight_spacing)
		.row_highlight_style(theme.row_highlight_style())
		.highlight_symbol(HIGHLIGHT_SYMBOL);
	frame.render_stateful_widget(table, area, table_state);

	render_header_separator(frame, area, theme, 1);
}

fn render_header_separator(frame: &mut Frame, area: Rect, theme: &Theme, header_height: u16) {
	if header_height >= area.height {
		return;
	}
	let sep_y = area.y + header_height;
	if sep_y >= area.y + area.height {
		return;
	}

	let width = area.width as usize;
	if width == 0 {
		return;
	}

	let sep_rect = Rect {
		x: area.x,
		y: sep_y,
		width: area.width,
		height: 1,
	};
	if width <= 2 {
		let line = " ".repeat(width);
		let para = Paragraph::new(line);
		frame.render_widget(para, sep_rect);
		return;
	}

	let middle = "─".repeat(width - 2);
	let middle_style = Style::default().fg(theme.header_fg());
	let middle_span = Span::styled(middle, middle_style);
	let spans = vec![Span::raw(" "), middle_span, Span::raw(" ")];
	let para = Paragraph::new(Text::from(Line::from(spans)));
	frame.render_widget(para, sep_rect);
}
