pub mod rows;

use self::rows::{build_facet_rows, build_file_rows};
use frizbee::Options;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Cell, HighlightSpacing, Paragraph, Row, Table};
use unicode_width::UnicodeWidthStr;

use crate::types::UiConfig;

/// Description of a table pane to render.
pub enum TablePane<'a> {
    Facets {
        filtered: &'a [usize],
        scores: &'a [u16],
        facets: &'a [crate::types::FacetRow],
        headers: Option<&'a Vec<String>>,
        widths: Option<&'a Vec<Constraint>>,
    },
    Files {
        filtered: &'a [usize],
        scores: &'a [u16],
        files: &'a [crate::types::FileRow],
        headers: Option<&'a Vec<String>>,
        widths: Option<&'a Vec<Constraint>>,
    },
}

/// Unified renderer for both kinds of tables. Accepts a `TablePane` which
/// packages all pane-specific data.
const HIGHLIGHT_SYMBOL: &str = "▶ ";
const TABLE_COLUMN_SPACING: u16 = 1;

pub fn render_table(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    table_state: &mut ratatui::widgets::TableState,
    _ui: &UiConfig,
    highlight_state: Option<(&str, Options)>,
    pane: TablePane<'_>,
    theme: &crate::theme::Theme,
) {
    let highlight_spacing = HighlightSpacing::WhenSelected;
    let selection_width = selection_column_width(table_state, &highlight_spacing);
    let pane_params = PaneParameters::from_pane(pane, area, selection_width, highlight_state);
    render_configured_table(
        frame,
        area,
        table_state,
        highlight_spacing,
        theme,
        pane_params,
    );
}

struct PaneParameters<'a> {
    widths: Vec<Constraint>,
    headers: Vec<String>,
    rows: Vec<Row<'a>>,
}

impl<'a> PaneParameters<'a> {
    fn from_pane(
        pane: TablePane<'a>,
        area: Rect,
        selection_width: u16,
        highlight_state: Option<(&'a str, Options)>,
    ) -> Self {
        match pane {
            TablePane::Facets {
                filtered,
                scores,
                facets,
                headers,
                widths,
            } => {
                let widths_owned = widths.cloned().unwrap_or_else(|| {
                    vec![
                        Constraint::Percentage(50),
                        Constraint::Length(8),
                        Constraint::Length(8),
                    ]
                });
                let column_widths = resolve_column_widths(
                    area,
                    &widths_owned,
                    selection_width,
                    TABLE_COLUMN_SPACING,
                );
                let rows = build_facet_rows(
                    filtered,
                    scores,
                    facets,
                    highlight_state,
                    Some(&column_widths),
                );
                let headers = headers
                    .cloned()
                    .unwrap_or_else(|| vec!["Facet".into(), "Count".into(), "Score".into()]);
                Self {
                    widths: widths_owned,
                    headers,
                    rows,
                }
            }
            TablePane::Files {
                filtered,
                scores,
                files,
                headers,
                widths,
            } => {
                let widths_owned = widths.cloned().unwrap_or_else(|| {
                    vec![
                        Constraint::Percentage(60),
                        Constraint::Percentage(30),
                        Constraint::Length(8),
                    ]
                });
                let column_widths = resolve_column_widths(
                    area,
                    &widths_owned,
                    selection_width,
                    TABLE_COLUMN_SPACING,
                );
                let rows = build_file_rows(
                    filtered,
                    scores,
                    files,
                    highlight_state,
                    Some(&column_widths),
                );
                let headers = headers
                    .cloned()
                    .unwrap_or_else(|| vec!["Path".into(), "Tags".into(), "Score".into()]);
                Self {
                    widths: widths_owned,
                    headers,
                    rows,
                }
            }
        }
    }
}

fn render_configured_table(
    frame: &mut Frame,
    area: Rect,
    table_state: &mut ratatui::widgets::TableState,
    highlight_spacing: HighlightSpacing,
    theme: &crate::theme::Theme,
    params: PaneParameters<'_>,
) {
    let header_cells = params
        .headers
        .into_iter()
        .map(Cell::from)
        .collect::<Vec<_>>();
    let header = Row::new(header_cells)
        .style(theme.header_style())
        .height(1)
        .bottom_margin(1);

    let table = Table::new(params.rows, params.widths)
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
    theme: &crate::theme::Theme,
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

fn resolve_column_widths(
    area: Rect,
    constraints: &[Constraint],
    selection_width: u16,
    column_spacing: u16,
) -> Vec<u16> {
    if constraints.is_empty() {
        return Vec::new();
    }

    let layout_area = Rect {
        x: 0,
        y: 0,
        width: area.width,
        height: 1,
    };
    let [_, columns_area] =
        Layout::horizontal([Constraint::Length(selection_width), Constraint::Fill(0)])
            .areas(layout_area);

    Layout::horizontal(constraints.to_vec())
        .spacing(column_spacing)
        .split(columns_area)
        .iter()
        .map(|rect| rect.width)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FacetRow, FileRow};

    fn mock_rect() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 10,
        }
    }

    #[test]
    fn facets_pane_uses_default_configuration() {
        let filtered = Vec::<usize>::new();
        let scores = Vec::<u16>::new();
        let facets = Vec::<FacetRow>::new();
        let pane = TablePane::Facets {
            filtered: &filtered,
            scores: &scores,
            facets: &facets,
            headers: None,
            widths: None,
        };

        let params = PaneParameters::from_pane(pane, mock_rect(), 0, None);

        assert_eq!(params.headers, vec!["Facet", "Count", "Score"]);
        assert_eq!(
            params.widths,
            vec![
                Constraint::Percentage(50),
                Constraint::Length(8),
                Constraint::Length(8),
            ]
        );
        assert!(params.rows.is_empty());
    }

    #[test]
    fn files_pane_respects_custom_configuration() {
        let filtered = vec![0usize];
        let scores = vec![42u16];
        let files = vec![FileRow::new("file.txt", Vec::<String>::new())];
        let headers = vec!["One".to_string(), "Two".to_string(), "Three".to_string()];
        let widths = vec![
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
        ];
        let pane = TablePane::Files {
            filtered: &filtered,
            scores: &scores,
            files: &files,
            headers: Some(&headers),
            widths: Some(&widths),
        };

        let params = PaneParameters::from_pane(pane, mock_rect(), 0, None);

        assert_eq!(params.headers, headers);
        assert_eq!(params.widths, widths);
        assert_eq!(params.rows.len(), filtered.len());
    }
}
