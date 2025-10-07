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
    render_progress(frame, input_area, progress_text, throbber_state, theme);
    search_input.render_textarea(frame, input_area);

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
    throbber_state: &ThrobberState,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 || progress_text.is_empty() {
        return;
    }

    let muted_style = theme.empty_style();
    let spinner = Throbber::default()
        .style(muted_style)
        .throbber_style(muted_style);
    let spinner_span = spinner.to_symbol_span(throbber_state);
    let label_span = Span::styled(progress_text.to_string(), muted_style);
    let mut line = Line::default();
    line.spans.push(spinner_span);
    line.spans.push(label_span);

    let line_width = line.width() as u16;
    if line_width == 0 {
        return;
    }

    let max_width = area.width.min(line_width);
    let start_x = if line_width >= area.width {
        area.left()
    } else {
        area.right().saturating_sub(line_width)
    };

    frame
        .buffer_mut()
        .set_line(start_x, area.top(), &line, max_width);
}
