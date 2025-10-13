use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    widgets::{Clear, Paragraph},
};

use crate::search;
use crate::theme::Theme;
use crate::types::SearchMode;
use frizbee::Options;

use super::App;
use super::components::{
    InputContext, ProgressState, TablePane, render_input_with_tabs, render_table,
};

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
        self.render_results(frame, layout[1]);

        if self.filtered_len() == 0 {
            let empty = Paragraph::new("No results")
                .alignment(Alignment::Center)
                .style(Theme::default().empty_style());
            frame.render_widget(Clear, layout[1]);
            frame.render_widget(empty, layout[1]);
        }
    }

    fn progress_status(&mut self) -> (String, bool) {
        let facet_label = self.ui.facets.count_label.as_str();
        let file_label = self.ui.files.count_label.as_str();
        self.index_progress.status(facet_label, file_label)
    }

    fn render_results(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match self.mode {
            SearchMode::Facets => self.render_facets(frame, area),
            SearchMode::Files => self.render_files(frame, area),
        }
    }

    fn render_facets(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let highlight_owned = self.highlight_for_query(self.data.facets.len());
        let highlight_state = highlight_owned
            .as_ref()
            .map(|(text, config)| (text.as_str(), *config));
        render_table(
            frame,
            area,
            &mut self.table_state,
            &self.ui,
            highlight_state,
            TablePane::Facets {
                filtered: &self.filtered_facets,
                scores: &self.facet_scores,
                facets: &self.data.facets,
                headers: self.facet_headers.as_ref(),
                widths: self.facet_widths.as_ref(),
            },
            &self.theme,
        )
    }

    fn render_files(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let highlight_owned = self.highlight_for_query(self.data.files.len());
        let highlight_state = highlight_owned
            .as_ref()
            .map(|(text, config)| (text.as_str(), *config));
        render_table(
            frame,
            area,
            &mut self.table_state,
            &self.ui,
            highlight_state,
            TablePane::Files {
                filtered: &self.filtered_files,
                scores: &self.file_scores,
                files: &self.data.files,
                headers: self.file_headers.as_ref(),
                widths: self.file_widths.as_ref(),
            },
            &self.theme,
        )
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
