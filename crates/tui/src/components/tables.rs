use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
	Block, Borders, Cell, HighlightSpacing, Paragraph, Row, ScrollbarState, Table,
};

use crate::components::render_scrollbar;
use crate::style::Theme;

pub(crate) const HIGHLIGHT_SYMBOL: &str = "▶ ";
pub(crate) const TABLE_COLUMN_SPACING: u16 = 1;
pub(crate) const TABLE_HIGHLIGHT_SPACING: HighlightSpacing = HighlightSpacing::WhenSelected;
/// Header row + separator height inside the table's viewport.
pub(crate) const TABLE_HEADER_ROWS: usize = 2;

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
	/// Spacing to use for row highlighting.
	pub highlight_spacing: HighlightSpacing,
}

/// Render the table using the provided dataset definition.
pub fn render_table(
	frame: &mut Frame,
	area: Rect,
	table_state: &mut ratatui::widgets::TableState,
	scrollbar_state: &mut ScrollbarState,
	scrollbar_area: &mut Option<Rect>,
	spec: TableSpec<'_>,
	theme: &Theme,
) {
	*scrollbar_area = None;

	let mut block = Block::default()
		.borders(Borders::ALL)
		.border_set(ratatui::symbols::border::ROUNDED)
		.border_style(Style::default().fg(theme.header.fg.unwrap_or(ratatui::style::Color::Reset)));

	if let Some(title) = spec.title.clone() {
		block = block.title(title);
	}

	let inner = block.inner(area);
	frame.render_widget(block, area);

	render_configured_table(
		frame,
		inner,
		table_state,
		scrollbar_state,
		scrollbar_area,
		theme,
		spec,
	);
}

fn render_configured_table(
	frame: &mut Frame,
	area: Rect,
	table_state: &mut ratatui::widgets::TableState,
	scrollbar_state: &mut ScrollbarState,
	scrollbar_area: &mut Option<Rect>,
	theme: &Theme,
	spec: TableSpec<'_>,
) {
	let header_cells = spec.headers.into_iter().map(Cell::from).collect::<Vec<_>>();
	let header_style = Style::default().fg(theme.header.fg.unwrap_or(ratatui::style::Color::Reset));
	let header = Row::new(header_cells)
		.style(header_style)
		.height(1)
		.bottom_margin(1);

	let mut widths = spec.widths;
	if widths.is_empty() {
		widths = vec![Constraint::Fill(1)];
	}

	// Calculate viewport height (header + separator + visible rows)
	let header_height = TABLE_HEADER_ROWS;
	let viewport_height = area.height as usize;
	let available_rows = viewport_height.saturating_sub(header_height);
	let total_rows = spec.rows.len();

	// Determine if we need a scrollbar
	let needs_scrollbar = total_rows > available_rows && available_rows > 0;

	// Adjust table area if scrollbar is needed
	let table_area = if needs_scrollbar {
		Rect {
			x: area.x,
			y: area.y,
			width: area.width.saturating_sub(1),
			height: area.height,
		}
	} else {
		area
	};

	// Render table
	let table = Table::new(spec.rows, widths)
		.header(header)
		.column_spacing(TABLE_COLUMN_SPACING)
		.highlight_spacing(spec.highlight_spacing)
		.row_highlight_style(theme.row_highlight)
		.highlight_symbol(HIGHLIGHT_SYMBOL);
	frame.render_stateful_widget(table, table_area, table_state);

	// Render scrollbar if needed
	if needs_scrollbar {
		render_scrollbar(frame, area, scrollbar_state, scrollbar_area, theme);
	}

	render_header_separator(frame, table_area, theme, 1);
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
	let middle_style = Style::default().fg(theme.header.fg.unwrap_or(ratatui::style::Color::Reset));
	let middle_span = Span::styled(middle, middle_style);
	let spans = vec![Span::raw(" "), middle_span, Span::raw(" ")];
	let para = Paragraph::new(Text::from(Line::from(spans)));
	frame.render_widget(para, sep_rect);
}
