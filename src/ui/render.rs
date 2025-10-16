use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    widgets::{Clear, Paragraph},
};

use crate::systems::search;
use crate::theme::Theme;
use frizbee::Options;

use super::App;
use super::components::{InputContext, ProgressState, render_input_with_tabs};

impl<'a> App<'a> {
    pub(crate) fn draw(&mut self, frame: &mut Frame) {
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
        let input_ctx = InputContext {
            search_input: &self.search_input,
            input_title: &self.input_title,
            mode: self.mode,
            ui: &self.ui,
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
        self.render_results(frame, results_area);

        if self.filtered_len() == 0 {
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
        let modes = self.plugin_modes();
        self.index_progress.status(&self.ui, modes)
    }

    fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        if let Some(definition) = self.plugin_definition(self.mode) {
            (definition.behavior.render)(self, frame, area);
        }
    }

    pub(crate) fn highlight_for_query(&self, dataset_len: usize) -> Option<(String, Options)> {
        let query = self.search_input.text().trim();
        if query.is_empty() {
            return None;
        }
        let config = search::config_for_query(query, dataset_len);
        Some((query.to_string(), config))
    }
}
