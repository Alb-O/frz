use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph},
};

use crate::extensions::api::{
    Icon, PreviewLayout, PreviewSplitContext, PreviewSplitStore, SelectionResolverStore,
};
use crate::logging;
use crate::systems::search;
use crate::tui::components::{
    InputContext, ProgressState, TabItem, TableRenderContext, render_input_with_tabs, render_table,
};
pub use crate::tui::theme::Theme;
use frizbee::Options;

use super::App;

impl<'a> App<'a> {
    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        logging::pump();

        let area = frame.area();
        let area = area.inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        let (progress_text, progress_complete) = self.progress_status();
        let tabs = self
            .ui
            .tabs()
            .iter()
            .map(|tab| TabItem {
                mode: tab.mode,
                label: tab.tab_label.as_str(),
            })
            .collect::<Vec<_>>();
        let input_ctx = InputContext {
            search_input: &self.search_input,
            input_title: self.input_title.as_deref(),
            pane_title: self.ui.pane(self.mode).map(|pane| pane.mode_title.as_str()),
            mode: self.mode,
            tabs: &tabs,
            area: layout[0],
            theme: &self.theme,
        };
        let progress_state = ProgressState {
            progress_text: &progress_text,
            progress_complete,
            throbber_state: &self.throbber_state,
        };
        render_input_with_tabs(frame, input_ctx, progress_state);
        let results_area = layout[1];
        let preview_layout = self.render_results(frame, results_area);

        if self.filtered_len() == 0 && preview_layout != PreviewLayout::PreviewOnly {
            let mut message_area = results_area;
            const HEADER_AND_DIVIDER_HEIGHT: u16 = 2;
            if message_area.height > HEADER_AND_DIVIDER_HEIGHT {
                message_area.y += HEADER_AND_DIVIDER_HEIGHT;
                message_area.height -= HEADER_AND_DIVIDER_HEIGHT;

                let empty = Paragraph::new("No results")
                    .alignment(Alignment::Center)
                    .style(Theme::default().empty_style());
                frame.render_widget(Clear, message_area);
                frame.render_widget(empty, message_area);
            }
        }
    }

    fn progress_status(&mut self) -> (String, bool) {
        let mut labels = Vec::new();
        for tab in self.ui.tabs() {
            labels.push((tab.mode.id(), tab.pane.count_label.clone()));
        }
        self.index_progress.status(&labels)
    }

    fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) -> PreviewLayout {
        let descriptor = self.mode.descriptor();
        let dataset = descriptor.dataset;
        let scope = self.contributions().scope(self.mode);
        let preview_split = scope.resolve::<PreviewSplitStore>();
        let selection_resolver = scope.resolve::<SelectionResolverStore>();
        let preview_icon = preview_split.as_ref().and_then(|split| split.header_icon());
        let query = self.search_input.text().to_string();

        const HEADER_HEIGHT: u16 = 1;
        let capability_title = self.ui.mode_table_title(self.mode).to_string();
        let preview_layout = preview_split
            .as_ref()
            .map(|split| split.layout())
            .unwrap_or(PreviewLayout::Split);

        let mut table_area_opt = Some(area);
        let mut preview_header_area = None;
        let mut preview_content_area = None;

        match preview_layout {
            PreviewLayout::Split => {
                if preview_split.is_some() {
                    let [table_area, preview_area] = Layout::horizontal([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .areas(area);
                    table_area_opt = Some(table_area);

                    if preview_area.height > 0 {
                        let header_height = HEADER_HEIGHT.min(preview_area.height);
                        preview_header_area = Some(Rect {
                            x: preview_area.x,
                            y: preview_area.y,
                            width: preview_area.width,
                            height: header_height,
                        });

                        let total_width = preview_area
                            .x
                            .saturating_add(preview_area.width)
                            .saturating_sub(table_area.x);
                        let header_rect = Rect {
                            x: table_area.x,
                            y: table_area.y,
                            width: total_width,
                            height: header_height,
                        };

                        render_split_header_background(frame, header_rect, &self.theme);

                        let content_height = preview_area.height.saturating_sub(header_height);
                        if content_height > 0 {
                            preview_content_area = Some(Rect {
                                x: preview_area.x,
                                y: preview_area.y + header_height,
                                width: preview_area.width,
                                height: content_height,
                            });
                        }
                    }
                } else {
                    preview_content_area = None;
                }
            }
            PreviewLayout::PreviewOnly => {
                table_area_opt = None;
                if preview_split.is_some() && area.height > 0 {
                    let header_height = HEADER_HEIGHT.min(area.height);
                    preview_header_area = Some(Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: header_height,
                    });

                    let header_rect = Rect {
                        x: area.x,
                        y: area.y,
                        width: area.width,
                        height: header_height,
                    };
                    render_split_header_background(frame, header_rect, &self.theme);

                    let content_height = area.height.saturating_sub(header_height);
                    if content_height > 0 {
                        preview_content_area = Some(Rect {
                            x: area.x,
                            y: area.y + header_height,
                            width: area.width,
                            height: content_height,
                        });
                    }
                }
            }
        }

        let highlight_owned = self.highlight_for_query(dataset.total_count(&self.data));
        let highlight_state = highlight_owned
            .as_ref()
            .map(|(text, config)| (text.as_str(), *config));
        {
            let state = self.tab_states.entry(self.mode).or_default();
            if let Some(table_area) = table_area_opt {
                render_table(
                    frame,
                    table_area,
                    &mut self.table_state,
                    dataset,
                    &self.theme,
                    TableRenderContext {
                        area: table_area,
                        filtered: &state.filtered,
                        scores: &state.scores,
                        headers: state.headers.as_ref(),
                        widths: state.widths.as_ref(),
                        highlight: highlight_state,
                        scope: scope.clone(),
                        data: &self.data,
                    },
                );
            }

            if let (Some(preview), Some(preview_area)) =
                (preview_split.clone(), preview_content_area)
            {
                let selected = self.table_state.selected();
                let selection_resource = selection_resolver
                    .as_ref()
                    .and_then(|resolver| resolver.resolve(&self.data, &state.filtered, selected));
                let context = PreviewSplitContext::new(
                    &self.data,
                    &state.filtered,
                    &state.scores,
                    selected,
                    selection_resource,
                    query.as_str(),
                    self.bat_theme.as_deref(),
                    self.git_modifications,
                );
                preview.render_preview(frame, preview_area, context);
            }
        }

        if let Some(header_area) = preview_header_area {
            render_split_header_title(
                frame,
                header_area,
                &self.theme,
                &capability_title,
                preview_icon,
            );
            if let Some(table_area) = table_area_opt {
                render_split_header_separator(frame, table_area, header_area, &self.theme);
            }
        }

        preview_layout
    }

    fn highlight_for_query(&self, dataset_len: usize) -> Option<(String, Options)> {
        let query = self.search_input.text().trim();
        if query.is_empty() {
            return None;
        }
        let config = search::config_for_query(query, dataset_len);
        Some((query.to_string(), config))
    }
}

fn render_split_header_background(frame: &mut Frame, header_rect: Rect, theme: &Theme) {
    if header_rect.width == 0 || header_rect.height == 0 {
        return;
    }

    let fill = " ".repeat(header_rect.width as usize);
    let background = Paragraph::new(fill).style(Style::new().bg(theme.header_bg()));
    frame.render_widget(background, header_rect);
}

fn render_split_header_separator(
    frame: &mut Frame,
    table_area: Rect,
    preview_header_area: Rect,
    theme: &Theme,
) {
    let separator_y = table_area.y.saturating_add(1);
    if separator_y >= table_area.y.saturating_add(table_area.height) {
        return;
    }

    let width = preview_header_area
        .x
        .saturating_add(preview_header_area.width)
        .saturating_sub(table_area.x);
    if width == 0 {
        return;
    }

    let separator_rect = Rect {
        x: table_area.x,
        y: separator_y,
        width,
        height: 1,
    };

    let header_bg = theme.header_bg();
    let base_style = Style::new().bg(header_bg);
    let width_usize = separator_rect.width as usize;
    if width_usize <= 2 {
        let line = " ".repeat(width_usize);
        let para = Paragraph::new(line).style(base_style);
        frame.render_widget(para, separator_rect);
        return;
    }

    let middle = "─".repeat(width_usize - 2);
    let middle_style = Style::new().bg(header_bg).fg(theme.header_fg());
    let spans = vec![
        Span::styled(" ", base_style),
        Span::styled(middle, middle_style),
        Span::styled(" ", base_style),
    ];
    let para = Paragraph::new(Text::from(Line::from(spans)));
    frame.render_widget(para, separator_rect);
}

fn render_split_header_title(
    frame: &mut Frame,
    header_area: Rect,
    theme: &Theme,
    capability: &str,
    icon: Option<Icon>,
) {
    if (capability.is_empty() && icon.is_none())
        || header_area.width == 0
        || header_area.height == 0
    {
        return;
    }

    let mut spans = Vec::new();
    if let Some(icon) = icon {
        let mut span = icon.to_padded_span();
        span.style = span.style.bg(theme.header_bg());
        spans.push(span);
    }

    if !capability.is_empty() {
        spans.push(Span::styled(capability.to_owned(), theme.header_style()));
    }

    let title = Paragraph::new(Text::from(Line::from(spans)))
        .alignment(Alignment::Center)
        .style(theme.header_style());
    frame.render_widget(title, header_area);
}
