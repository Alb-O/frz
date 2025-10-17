use crate::plugins::api::SearchMode;
use crate::tui::input::SearchInput;
use crate::tui::theme::Theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use throbber_widgets_tui::{Throbber, ThrobberState};

/// Render metadata for a tab header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabItem<'a> {
    pub mode: SearchMode,
    pub label: &'a str,
}

/// Argument bundle for rendering the input area.
pub struct InputContext<'a> {
    pub search_input: &'a SearchInput<'a>,
    pub input_title: Option<&'a str>,
    pub pane_title: Option<&'a str>,
    pub mode: SearchMode,
    pub tabs: &'a [TabItem<'a>],
    pub area: Rect,
    pub theme: &'a Theme,
}

/// Progress information for the prompt progress indicator.
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
        pane_title,
        mode,
        tabs,
        area,
        theme,
    } = input;
    let ProgressState {
        progress_text,
        progress_complete,
        throbber_state,
    } = progress;

    let prompt = input_title.or(pane_title).unwrap_or("");
    let tabs_width = calculate_tabs_width(tabs);
    let prompt_width = calculate_prompt_width(prompt);

    let constraints = layout_constraints(!prompt.is_empty(), prompt_width, tabs_width);

    let horizontal = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(constraints)
        .split(area);

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

    let tabs_area = horizontal[horizontal.len() - 1];
    let tabs_inner = Rect {
        x: tabs_area.x.saturating_add(1),
        width: tabs_area.width.saturating_sub(1),
        ..tabs_area
    };
    let selected = selected_tab_index(mode, tabs);

    let tab_titles = build_tab_titles(theme, selected, tabs);

    let tabs = Tabs::new(tab_titles)
        .select(selected)
        .divider("")
        .padding("", " ")
        .highlight_style(theme.tab_highlight_style());

    frame.render_widget(tabs, tabs_inner);
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

fn selected_tab_index(mode: SearchMode, tabs: &[TabItem<'_>]) -> usize {
    tabs.iter().position(|tab| tab.mode == mode).unwrap_or(0)
}

fn build_tab_titles(theme: &Theme, selected: usize, tabs: &[TabItem<'_>]) -> Vec<Line<'static>> {
    let active = theme.header_style();
    let inactive = theme.tab_inactive_style();
    tabs.iter()
        .enumerate()
        .map(|(index, tab)| {
            let label = format!(" {} ", tab.label);
            let style = if index == selected { active } else { inactive };
            Line::from(label).style(style)
        })
        .collect()
}

fn calculate_tabs_width(tabs: &[TabItem<'_>]) -> u16 {
    let mut width = 0u16;
    for tab in tabs {
        let label_len = tab.label.chars().count() as u16;
        width = width.saturating_add(label_len.saturating_add(3));
    }
    width.max(12)
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
        let min_start = last_x.saturating_add(3);
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
    use crate::plugins::api::{
        SearchData, SearchMode,
        descriptors::{
            SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
            TableDescriptor,
        },
    };
    use ratatui::{Terminal, backend::TestBackend};
    use throbber_widgets_tui::ThrobberState;

    struct DummyDataset;

    impl SearchPluginDataset for DummyDataset {
        fn key(&self) -> &'static str {
            "dummy"
        }

        fn total_count(&self, _data: &SearchData) -> usize {
            0
        }

        fn build_table<'a>(&self, _context: TableContext<'a>) -> TableDescriptor<'a> {
            TableDescriptor::new(Vec::new(), Vec::new(), Vec::new())
        }
    }

    static DATASET: DummyDataset = DummyDataset;

    static TAG_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
        id: "attributes",
        ui: SearchPluginUiDefinition {
            tab_label: "Tags",
            mode_title: "Tag search",
            hint: "",
            table_title: "",
            count_label: "",
        },
        dataset: &DATASET,
    };

    static FILE_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
        id: "files",
        ui: SearchPluginUiDefinition {
            tab_label: "Files",
            mode_title: "File search",
            hint: "",
            table_title: "",
            count_label: "",
        },
        dataset: &DATASET,
    };

    static OTHER_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
        id: "other",
        ui: SearchPluginUiDefinition {
            tab_label: "Other",
            mode_title: "Other search",
            hint: "",
            table_title: "",
            count_label: "",
        },
        dataset: &DATASET,
    };

    fn mode(descriptor: &'static SearchPluginDescriptor) -> SearchMode {
        SearchMode::from_descriptor(descriptor)
    }

    #[test]
    fn prompt_width_accounts_for_separator() {
        assert_eq!(calculate_prompt_width(""), 0);
        assert_eq!(calculate_prompt_width("Prompt"), 9);
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
        let tabs = vec![
            TabItem {
                mode: mode(&TAG_DESCRIPTOR),
                label: TAG_DESCRIPTOR.ui.tab_label,
            },
            TabItem {
                mode: mode(&FILE_DESCRIPTOR),
                label: FILE_DESCRIPTOR.ui.tab_label,
            },
        ];
        assert_eq!(selected_tab_index(mode(&TAG_DESCRIPTOR), &tabs), 0);
        assert_eq!(selected_tab_index(mode(&FILE_DESCRIPTOR), &tabs), 1);
        let other = SearchMode::from_descriptor(&OTHER_DESCRIPTOR);
        assert_eq!(selected_tab_index(other, &tabs), 0);
    }

    #[test]
    fn tab_titles_include_expected_labels() {
        let theme = Theme::default();
        let tabs = vec![
            TabItem {
                mode: mode(&TAG_DESCRIPTOR),
                label: TAG_DESCRIPTOR.ui.tab_label,
            },
            TabItem {
                mode: mode(&FILE_DESCRIPTOR),
                label: FILE_DESCRIPTOR.ui.tab_label,
            },
        ];
        let titles = build_tab_titles(&theme, 0, &tabs);

        assert_eq!(titles.len(), 2);
        assert_eq!(titles[0].spans[0].content.as_ref().trim(), "Tags");
        assert_eq!(titles[1].spans[0].content.as_ref().trim(), "Files");
        assert_eq!(titles[0].style, theme.header_style());
        assert_eq!(titles[1].style, theme.tab_inactive_style());
    }

    #[test]
    fn tabs_width_accounts_for_padding() {
        let tabs = vec![
            TabItem {
                mode: mode(&TAG_DESCRIPTOR),
                label: TAG_DESCRIPTOR.ui.tab_label,
            },
            TabItem {
                mode: mode(&FILE_DESCRIPTOR),
                label: FILE_DESCRIPTOR.ui.tab_label,
            },
        ];
        assert!(calculate_tabs_width(&tabs) >= 12);
    }

    #[test]
    fn rendering_input_with_tabs_populates_buffer() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        let input = SearchInput::new("hello");
        let tabs = vec![
            TabItem {
                mode: mode(&TAG_DESCRIPTOR),
                label: TAG_DESCRIPTOR.ui.tab_label,
            },
            TabItem {
                mode: mode(&FILE_DESCRIPTOR),
                label: FILE_DESCRIPTOR.ui.tab_label,
            },
        ];
        let theme = Theme::default();
        let throbber_state = ThrobberState::default();
        let current_mode = mode(&FILE_DESCRIPTOR);

        terminal
            .draw(|frame| {
                let area = frame.area();
                let context = InputContext {
                    search_input: &input,
                    input_title: Some("Search"),
                    pane_title: None,
                    mode: current_mode,
                    tabs: &tabs,
                    area,
                    theme: &theme,
                };
                let progress = ProgressState {
                    progress_text: "Indexing files",
                    progress_complete: true,
                    throbber_state: &throbber_state,
                };
                render_input_with_tabs(frame, context, progress);
            })
            .expect("render frame");

        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        let first_row = buffer
            .content
            .chunks(width)
            .next()
            .expect("first row available");
        let rendered: String = first_row.iter().map(|cell| cell.symbol()).collect();

        assert!(rendered.contains("Search"));
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("Indexing files"));
    }
}
