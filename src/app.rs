use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    widgets::{Clear, Paragraph, TableState},
};

use crate::indexing::{IndexUpdate, merge_update};
use crate::input::SearchInput;
use crate::progress::IndexProgress;
use crate::tables;
use crate::tabs;
use crate::theme::Theme;
use crate::types::{SearchData, SearchMode, SearchOutcome, SearchSelection, UiConfig};
use frizbee::Config;
use throbber_widgets_tui::ThrobberState;

use crate::search::{self, SearchCommand, SearchResult};
pub fn run(data: SearchData) -> Result<SearchOutcome> {
    let mut app: App = App::new(data);
    app.run()
}

impl<'a> Drop for App<'a> {
    fn drop(&mut self) {
        let _ = self.search_tx.send(SearchCommand::Shutdown);
    }
}

pub struct App<'a> {
    pub data: SearchData,
    pub mode: SearchMode,
    pub search_input: SearchInput<'a>,
    pub table_state: TableState,
    pub filtered_facets: Vec<usize>,
    pub filtered_files: Vec<usize>,
    pub facet_scores: Vec<u16>,
    pub file_scores: Vec<u16>,
    // Customization points for the API
    pub(crate) input_title: Option<String>,
    pub(crate) facet_headers: Option<Vec<String>>,
    pub(crate) file_headers: Option<Vec<String>>,
    pub(crate) facet_widths: Option<Vec<Constraint>>,
    pub(crate) file_widths: Option<Vec<Constraint>>,
    pub(crate) ui: UiConfig,
    pub theme: crate::theme::Theme,
    throbber_state: ThrobberState,
    index_progress: IndexProgress,
    index_updates: Option<Receiver<IndexUpdate>>,
    search_tx: Sender<SearchCommand>,
    search_rx: Receiver<SearchResult>,
    search_latest_query_id: Arc<AtomicU64>,
    next_query_id: u64,
    latest_query_id: Option<u64>,
}

impl<'a> App<'a> {
    pub fn new(data: SearchData) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let initial_query = data.initial_query.clone();
        let context_label = data.context_label.clone();
        let index_progress = IndexProgress::from(&data);
        let (search_tx, search_rx, search_latest_query_id) = search::spawn(data.clone());
        let mut app = Self {
            data,
            mode: SearchMode::Facets,
            search_input: SearchInput::new(initial_query),
            table_state,
            filtered_facets: Vec::new(),
            filtered_files: Vec::new(),
            facet_scores: Vec::new(),
            file_scores: Vec::new(),
            input_title: context_label,
            facet_headers: None,
            file_headers: None,
            facet_widths: None,
            file_widths: None,
            ui: UiConfig::default(),
            theme: Theme::default(),
            throbber_state: ThrobberState::default(),
            index_progress,
            index_updates: None,
            search_tx,
            search_rx,
            search_latest_query_id,
            next_query_id: 0,
            latest_query_id: None,
        };
        app.request_search();
        app.pump_search_results();
        app
    }

    /// Set the active theme for the app.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_mode(&mut self, mode: SearchMode) {
        if self.mode != mode {
            self.mode = mode;
            self.table_state.select(Some(0));
            self.request_search();
        }
    }

    /// Run the interactive application. This is a method so callers can
    /// customize `App` fields before launching (used by the `Searcher`
    /// builder in the crate root).
    pub fn run(&mut self) -> Result<SearchOutcome> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        let result = loop {
            self.pump_index_updates();
            self.pump_search_results();
            self.throbber_state.calc_next();
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if let Some(outcome) = self.handle_key(key)? {
                            break outcome;
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        };

        ratatui::restore();
        Ok(result)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let area = area.inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        // Input/tabs row (top line) and results below
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Delegate input + tabs rendering
        let (progress_text, progress_complete) = self.progress_status();
        tabs::render_input_with_tabs(
            &self.search_input,
            &self.input_title,
            self.mode,
            &self.ui,
            frame,
            layout[0],
            &self.theme,
            &progress_text,
            progress_complete,
            &self.throbber_state,
        );
        self.render_results(frame, layout[1]);

        // Minimal empty state
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
            SearchMode::Facets => {
                let query = self.search_input.text().trim();
                let highlight_owned = if query.is_empty() {
                    None
                } else {
                    let dataset_len = self.data.facets.len();
                    Some((
                        query.to_string(),
                        search::config_for_query(query, dataset_len),
                    ))
                };
                let highlight_state: Option<(&str, &Config)> =
                    highlight_owned.as_ref().map(|(s, c)| (s.as_str(), c));
                tables::render_table(
                    frame,
                    area,
                    &mut self.table_state,
                    &self.ui,
                    highlight_state,
                    tables::TablePane::Facets {
                        filtered: &self.filtered_facets,
                        scores: &self.facet_scores,
                        facets: &self.data.facets,
                        headers: self.facet_headers.as_ref(),
                        widths: self.facet_widths.as_ref(),
                    },
                    &self.theme,
                )
            }
            SearchMode::Files => {
                let query = self.search_input.text().trim();
                let highlight_owned = if query.is_empty() {
                    None
                } else {
                    let dataset_len = self.data.files.len();
                    Some((
                        query.to_string(),
                        search::config_for_query(query, dataset_len),
                    ))
                };
                let highlight_state: Option<(&str, &Config)> =
                    highlight_owned.as_ref().map(|(s, c)| (s.as_str(), c));
                tables::render_table(
                    frame,
                    area,
                    &mut self.table_state,
                    &self.ui,
                    highlight_state,
                    tables::TablePane::Files {
                        filtered: &self.filtered_files,
                        scores: &self.file_scores,
                        files: &self.data.files,
                        headers: self.file_headers.as_ref(),
                        widths: self.file_widths.as_ref(),
                    },
                    &self.theme,
                )
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<Option<SearchOutcome>> {
        match key.code {
            KeyCode::Esc => {
                return Ok(Some(SearchOutcome {
                    accepted: false,
                    selection: None,
                    query: self.search_input.text().to_string(),
                }));
            }
            KeyCode::Enter => {
                let selection = self.current_selection();
                return Ok(Some(SearchOutcome {
                    accepted: true,
                    selection,
                    query: self.search_input.text().to_string(),
                }));
            }
            KeyCode::Tab => {
                self.switch_mode();
            }
            KeyCode::Up => {
                self.move_selection_up();
            }
            KeyCode::Down => {
                self.move_selection_down();
            }
            _ => {
                // Let SearchInput handle all keys including arrow keys (for cursor movement), typing, backspace, etc.
                if self.search_input.input(key) {
                    self.request_search();
                }
            }
        }
        Ok(None)
    }

    fn switch_mode(&mut self) {
        self.mode = match self.mode {
            SearchMode::Facets => SearchMode::Files,
            SearchMode::Files => SearchMode::Facets,
        };
        self.table_state.select(Some(0));
        self.request_search();
    }

    fn move_selection_up(&mut self) {
        if let Some(selected) = self.table_state.selected()
            && selected > 0
        {
            self.table_state.select(Some(selected - 1));
        }
    }

    fn move_selection_down(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let len = self.filtered_len();
            if selected + 1 < len {
                self.table_state.select(Some(selected + 1));
            }
        }
    }

    fn current_selection(&self) -> Option<SearchSelection> {
        let selected = self.table_state.selected()?;
        match self.mode {
            SearchMode::Facets => {
                let index = *self.filtered_facets.get(selected)?;
                self.data
                    .facets
                    .get(index)
                    .cloned()
                    .map(SearchSelection::Facet)
            }
            SearchMode::Files => {
                let index = *self.filtered_files.get(selected)?;
                self.data
                    .files
                    .get(index)
                    .cloned()
                    .map(SearchSelection::File)
            }
        }
    }

    fn filtered_len(&self) -> usize {
        match self.mode {
            SearchMode::Facets => self.filtered_facets.len(),
            SearchMode::Files => self.filtered_files.len(),
        }
    }

    fn ensure_selection(&mut self) {
        if self.filtered_len() == 0 {
            self.table_state.select(None);
        } else if self.table_state.selected().is_none() {
            self.table_state.select(Some(0));
        } else if let Some(selected) = self.table_state.selected() {
            let len = self.filtered_len();
            if selected >= len {
                self.table_state.select(Some(len.saturating_sub(1)));
            }
        }
    }

    fn request_search(&mut self) {
        self.next_query_id = self.next_query_id.saturating_add(1);
        let id = self.next_query_id;
        self.latest_query_id = Some(id);
        let query = self.search_input.text().to_string();
        let mode = self.mode;
        self.search_latest_query_id
            .store(id, AtomicOrdering::Release);
        let _ = self
            .search_tx
            .send(SearchCommand::Query { id, query, mode });
    }

    fn pump_search_results(&mut self) {
        loop {
            match self.search_rx.try_recv() {
                Ok(result) => self.handle_search_result(result),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    fn handle_search_result(&mut self, result: SearchResult) {
        if Some(result.id) != self.latest_query_id {
            return;
        }

        match result.mode {
            SearchMode::Facets => {
                self.filtered_facets = result.indices;
                self.facet_scores = result.scores;
            }
            SearchMode::Files => {
                self.filtered_files = result.indices;
                self.file_scores = result.scores;
            }
        }

        self.ensure_selection();
    }

    pub(crate) fn set_index_updates(&mut self, updates: Receiver<IndexUpdate>) {
        self.index_updates = Some(updates);
        self.index_progress = IndexProgress::with_unknown_totals();
        self.index_progress
            .record_indexed(self.data.facets.len(), self.data.files.len());
    }

    fn pump_index_updates(&mut self) {
        let Some(rx) = self.index_updates.take() else {
            return;
        };

        let mut should_request = false;
        let mut keep_receiver = true;
        loop {
            match rx.try_recv() {
                Ok(update) => {
                    let _ = self.search_tx.send(SearchCommand::Update(update.clone()));
                    should_request |= self.apply_index_update(update);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    keep_receiver = false;
                    break;
                }
            }
        }

        if keep_receiver {
            self.index_updates = Some(rx);
        }

        if should_request {
            self.request_search();
        }
    }

    fn apply_index_update(&mut self, update: IndexUpdate) -> bool {
        let changed = !update.files.is_empty() || !update.facets.is_empty();
        if changed {
            merge_update(&mut self.data, &update);
        }

        let progress = update.progress;
        self.index_progress
            .record_indexed(progress.indexed_facets, progress.indexed_files);
        self.index_progress
            .set_totals(progress.total_facets, progress.total_files);
        if progress.complete {
            self.index_progress.mark_complete();
        }

        changed
    }
}
