use frizbee::Options;
pub use frz_tui::tables::rows::*;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Cell, HighlightSpacing, Paragraph, Row, Table},
};
use unicode_width::UnicodeWidthStr;

use frz_plugin_api::SearchData;
use frz_plugin_api::descriptors::{SearchPluginDataset, TableContext, TableDescriptor};

const HIGHLIGHT_SYMBOL: &str = "▶ ";
const TABLE_COLUMN_SPACING: u16 = 1;

/// Argument bundle describing the data a table render should use.
pub struct TableRenderContext<'a> {
    pub area: Rect,
    pub filtered: &'a [usize],
    pub scores: &'a [u16],
    pub headers: Option<&'a Vec<String>>,
    pub widths: Option<&'a Vec<Constraint>>,
    pub highlight: Option<(&'a str, Options)>,
    pub data: &'a SearchData,
}

/// Render a plugin-backed table using the provided dataset definition.
pub fn render_table(
    frame: &mut Frame,
    area: Rect,
    table_state: &mut ratatui::widgets::TableState,
    dataset: &'static dyn SearchPluginDataset,
    theme: &frz_tui::theme::Theme,
    context: TableRenderContext<'_>,
) {
    let highlight_spacing = HighlightSpacing::WhenSelected;
    let selection_width = selection_column_width(table_state, &highlight_spacing);
    let descriptor = dataset.build_table(TableContext {
        area,
        filtered: context.filtered,
        scores: context.scores,
        headers: context.headers,
        widths: context.widths,
        highlight: context.highlight,
        selection_width,
        column_spacing: TABLE_COLUMN_SPACING,
        data: context.data,
    });
    render_configured_table(
        frame,
        area,
        table_state,
        highlight_spacing,
        theme,
        descriptor,
    );
}

fn render_configured_table(
    frame: &mut Frame,
    area: Rect,
    table_state: &mut ratatui::widgets::TableState,
    highlight_spacing: HighlightSpacing,
    theme: &frz_tui::theme::Theme,
    descriptor: TableDescriptor<'_>,
) {
    let header_cells = descriptor
        .headers
        .into_iter()
        .map(Cell::from)
        .collect::<Vec<_>>();
    let header = Row::new(header_cells)
        .style(theme.header_style())
        .height(1)
        .bottom_margin(1);

    let table = Table::new(descriptor.rows, descriptor.widths)
        .header(header)
        .column_spacing(TABLE_COLUMN_SPACING)
        .highlight_spacing(highlight_spacing)
        .row_highlight_style(theme.row_highlight_style())
        .highlight_symbol(HIGHLIGHT_SYMBOL);
    frame.render_stateful_widget(table, area, table_state);

    render_header_separator(frame, area, theme, 1);
}

fn render_header_separator(
    frame: &mut Frame,
    area: Rect,
    theme: &frz_tui::theme::Theme,
    header_height: u16,
) {
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
    let header_bg = theme.header_bg();
    let base_style = Style::new().bg(header_bg);
    if width <= 2 {
        let line = " ".repeat(width);
        let para = Paragraph::new(line).style(base_style);
        frame.render_widget(para, sep_rect);
        return;
    }

    let middle = "─".repeat(width - 2);
    let middle_style = Style::new().bg(header_bg).fg(theme.header_fg());
    let middle_span = Span::styled(middle, middle_style);
    let spans = vec![
        Span::styled(" ", base_style),
        middle_span,
        Span::styled(" ", base_style),
    ];
    let para = Paragraph::new(Text::from(Line::from(spans)));
    frame.render_widget(para, sep_rect);
}

fn selection_column_width(state: &ratatui::widgets::TableState, spacing: &HighlightSpacing) -> u16 {
    let has_selection = state.selected().is_some();
    let should_add = match spacing {
        HighlightSpacing::Always => true,
        HighlightSpacing::WhenSelected => has_selection,
        HighlightSpacing::Never => false,
    };
    if should_add {
        UnicodeWidthStr::width(HIGHLIGHT_SYMBOL) as u16
    } else {
        0
    }
}
