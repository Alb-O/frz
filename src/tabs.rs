use crate::input::SearchInput;
use crate::theme::Theme;
use crate::types::SearchMode;
use crate::types::UiConfig;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use throbber_widgets_tui::{Throbber, ThrobberState};

/// Render the input row with tabs at the right. This mirrors the behaviour
/// previously implemented inside `app.rs`.
pub fn render_input_with_tabs(
    search_input: &SearchInput<'_>,
    input_title: &Option<String>,
    mode: SearchMode,
    ui: &UiConfig,
    frame: &mut ratatui::Frame,
    area: Rect,
    theme: &Theme,
    progress_text: &str,
    progress_complete: bool,
    throbber_state: &ThrobberState,
) {
    // Calculate tabs width: " Tags " + " Files " + extra padding = about 16 chars
    let tabs_width = 16u16;

    // Get prompt for calculating textarea width
    let prompt = input_title
        .as_deref()
        .or(Some(ui.facets.mode_title.as_str()))
        .unwrap_or("");
    let prompt_width = if prompt.is_empty() {
        0
    } else {
        prompt.len() as u16 + 3
    }; // " > "

    // Split area: prompt (if any), textarea, tabs on right
    let constraints = if prompt.is_empty() {
        vec![
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(tabs_width),
        ]
    } else {
        vec![
            ratatui::layout::Constraint::Length(prompt_width),
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(tabs_width),
        ]
    };

    let horizontal = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    // Render prompt if present
    if !prompt.is_empty() {
        let prompt_text = format!("{} > ", prompt);
        let prompt_widget =
            ratatui::widgets::Paragraph::new(prompt_text).style(theme.prompt_style());
        frame.render_widget(prompt_widget, horizontal[0]);
    }

    let input_index = if prompt.is_empty() { 0 } else { 1 };
    let input_area = horizontal[input_index];
    search_input.render_textarea(frame, input_area);
    render_progress(
        frame,
        input_area,
        progress_text,
        progress_complete,
        throbber_state,
        theme,
    );

    // Render tabs on the right (last section)
    let tabs_area = horizontal[horizontal.len() - 1];
    let selected = match mode {
        SearchMode::Facets => 0,
        SearchMode::Files => 1,
    };

    // Add extra padding to rightmost tab to prevent cutoff
    let tab_titles = vec![
        Line::from(format!(" {} ", "Tags"))
            .fg(theme.header_fg)
            .bg(if selected == 0 {
                theme.header_bg
            } else {
                theme.row_highlight_bg
            }),
        Line::from(format!(" {} ", "Files "))
            .fg(theme.header_fg)
            .bg(if selected == 1 {
                theme.header_bg
            } else {
                theme.row_highlight_bg
            }),
    ];

    let tabs = Tabs::new(tab_titles)
        .select(selected)
        .divider("")
        .highlight_style(Style::default().bg(theme.header_bg));

    frame.render_widget(tabs, tabs_area);
}

fn render_progress(
    frame: &mut ratatui::Frame,
    area: Rect,
    progress_text: &str,
    progress_complete: bool,
    throbber_state: &ThrobberState,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 || progress_text.is_empty() {
        return;
    }

    let muted_style = theme.empty_style();
    let label_span = Span::styled(progress_text.to_string(), muted_style);
    let mut line = Line::default();
    if !progress_complete {
        let spinner = Throbber::default()
            .style(muted_style)
            .throbber_style(muted_style);
        let spinner_span = spinner.to_symbol_span(throbber_state);
        line.spans.push(spinner_span);
    }
    line.spans.push(label_span);

    let line_width = line.width() as u16;
    if line_width == 0 {
        return;
    }

    let buffer = frame.buffer_mut();
    let mut start_x = if line_width >= area.width {
        area.left()
    } else {
        area.right().saturating_sub(line_width)
    };

    let input_row = area.top();
    let mut last_char_x: Option<u16> = None;
    for x in area.left()..area.right() {
        if let Some(cell) = buffer.cell((x, input_row))
            && !cell.symbol().trim().is_empty()
        {
            last_char_x = Some(x);
        }
    }

    if let Some(last_x) = last_char_x {
        let min_start = last_x.saturating_add(3); // 1 column for the last char + 2 columns padding
        if min_start > start_x {
            start_x = min_start;
        }
    }

    if start_x >= area.right() {
        return;
    }

    let max_width = area
        .right()
        .saturating_sub(start_x)
        .min(line_width)
        .min(area.width);

    if max_width == 0 {
        return;
    }

    buffer.set_line(start_x, input_row, &line, max_width);
}
