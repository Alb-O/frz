use crate::input::SearchInput;
use crate::theme::Theme;
use crate::types::SearchMode;
use crate::types::UiConfig;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use throbber_widgets_tui::{Throbber, ThrobberState};

/// Argument bundle for rendering the input area
pub struct InputContext<'a> {
    pub search_input: &'a SearchInput<'a>,
    pub input_title: &'a Option<String>,
    pub mode: SearchMode,
    pub ui: &'a UiConfig,
    pub area: Rect,
    pub theme: &'a Theme,
}

/// Progress information for the prompt progress indicator
pub struct ProgressState<'a> {
    pub progress_text: &'a str,
    pub progress_complete: bool,
    pub throbber_state: &'a ThrobberState,
}

/// Render the input row with tabs at the right.
pub fn render_input_with_tabs(
    frame: &mut ratatui::Frame,
    input: InputContext<'_>,
    progress: ProgressState<'_>,
) {
    let InputContext {
        search_input,
        input_title,
        mode,
        ui,
        area,
        theme,
    } = input;
    let ProgressState {
        progress_text,
        progress_complete,
        throbber_state,
    } = progress;
    // Calculate tabs width: " Tags " + " Files " + extra padding = about 16 chars
    let tabs_width = 16u16;

    // Get prompt for calculating textarea width
    let prompt = determine_prompt_text(input_title, ui);
    let prompt_width = calculate_prompt_width(prompt);

    // Split area: prompt (if any), textarea, tabs on right
    let constraints = layout_constraints(!prompt.is_empty(), prompt_width, tabs_width);

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
    let tabs_inner = Rect {
        x: tabs_area.x.saturating_add(1),
        width: tabs_area.width.saturating_sub(1),
        ..tabs_area
    };
    let selected = selected_tab_index(mode);

    // Add extra padding to rightmost tab to prevent cutoff
    let tab_titles = build_tab_titles(theme, selected);

    let tabs = Tabs::new(tab_titles)
        .select(selected)
        .divider("")
        .padding("", " ")
        .highlight_style(theme.tab_highlight_style());

    frame.render_widget(tabs, tabs_inner);
}

fn determine_prompt_text<'a>(input_title: &'a Option<String>, ui: &'a UiConfig) -> &'a str {
    input_title
        .as_deref()
        .or(Some(ui.facets.mode_title.as_str()))
        .unwrap_or("")
}

fn calculate_prompt_width(prompt: &str) -> u16 {
    if prompt.is_empty() {
        0
    } else {
        prompt.len() as u16 + 3
    }
}

fn layout_constraints(
    has_prompt: bool,
    prompt_width: u16,
    tabs_width: u16,
) -> Vec<ratatui::layout::Constraint> {
    if has_prompt {
        vec![
            ratatui::layout::Constraint::Length(prompt_width),
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(tabs_width),
        ]
    } else {
        vec![
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(tabs_width),
        ]
    }
}

fn selected_tab_index(mode: SearchMode) -> usize {
    match mode {
        SearchMode::Facets => 0,
        SearchMode::Files => 1,
    }
}

fn build_tab_titles(theme: &Theme, selected: usize) -> Vec<Line<'static>> {
    let active = theme.header_style();
    let inactive = theme.tab_inactive_style();
    vec![
        Line::from(format!(" {} ", "Tags")).style(if selected == 0 { active } else { inactive }),
        Line::from(format!(" {} ", "Files")).style(if selected == 1 { active } else { inactive }),
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_prefers_explicit_title() {
        let mut ui = UiConfig::default();
        ui.facets.mode_title = "Default".to_string();
        let input_title = Some("Custom".to_string());

        let prompt = determine_prompt_text(&input_title, &ui);

        assert_eq!(prompt, "Custom");
    }

    #[test]
    fn prompt_falls_back_to_ui_title() {
        let ui = UiConfig::default();
        let input_title = None;

        let prompt = determine_prompt_text(&input_title, &ui);

        assert_eq!(prompt, ui.facets.mode_title);
    }

    #[test]
    fn prompt_width_accounts_for_separator() {
        assert_eq!(calculate_prompt_width(""), 0);
        assert_eq!(calculate_prompt_width("Prompt"), 9); // len + " > "
    }

    #[test]
    fn layout_constraints_include_prompt_section() {
        let constraints = layout_constraints(true, 5, 10);

        assert_eq!(constraints.len(), 3);
        assert!(matches!(
            constraints[0],
            ratatui::layout::Constraint::Length(5)
        ));
        assert!(matches!(
            constraints[1],
            ratatui::layout::Constraint::Min(1)
        ));
        assert!(matches!(
            constraints[2],
            ratatui::layout::Constraint::Length(10)
        ));
    }

    #[test]
    fn layout_constraints_without_prompt_are_compact() {
        let constraints = layout_constraints(false, 5, 10);

        assert_eq!(constraints.len(), 2);
        assert!(matches!(
            constraints[0],
            ratatui::layout::Constraint::Min(1)
        ));
        assert!(matches!(
            constraints[1],
            ratatui::layout::Constraint::Length(10)
        ));
    }

    #[test]
    fn selected_tab_index_matches_mode() {
        assert_eq!(selected_tab_index(SearchMode::Facets), 0);
        assert_eq!(selected_tab_index(SearchMode::Files), 1);
    }

    #[test]
    fn tab_titles_include_expected_labels() {
        let theme = Theme::default();
        let titles = build_tab_titles(&theme, 0);

        assert_eq!(titles.len(), 2);
        assert_eq!(titles[0].spans[0].content.as_ref().trim(), "Tags");
        assert_eq!(titles[1].spans[0].content.as_ref().trim(), "Files");
        assert_eq!(titles[0].style, theme.header_style());
        assert_eq!(titles[1].style, theme.tab_inactive_style());
    }
}
