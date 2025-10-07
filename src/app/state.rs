use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::widgets::TableState;
use throbber_widgets_tui::ThrobberState;

use crate::indexing::IndexUpdate;
use crate::input::SearchInput;
use crate::progress::IndexProgress;
use crate::search::{self, SearchCommand, SearchResult};
use crate::theme::Theme;
use crate::types::{SearchData, SearchMode, SearchOutcome, SearchSelection, UiConfig};

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
    pub(crate) input_title: Option<String>,
    pub(crate) facet_headers: Option<Vec<String>>,
    pub(crate) file_headers: Option<Vec<String>>,
    pub(crate) facet_widths: Option<Vec<ratatui::layout::Constraint>>,
    pub(crate) file_widths: Option<Vec<ratatui::layout::Constraint>>,
    pub(crate) ui: UiConfig,
    pub theme: Theme,
    pub(crate) throbber_state: ThrobberState,
    pub(crate) index_progress: IndexProgress,
    pub(crate) index_updates: Option<Receiver<IndexUpdate>>,
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

    pub(crate) fn request_search(&mut self) {
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

    pub(crate) fn notify_search_of_update(&self, update: &IndexUpdate) {
        let _ = self.search_tx.send(SearchCommand::Update(update.clone()));
    }

    pub(crate) fn pump_search_results(&mut self) {
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

    pub(crate) fn ensure_selection(&mut self) {
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

    pub(crate) fn filtered_len(&self) -> usize {
        match self.mode {
            SearchMode::Facets => self.filtered_facets.len(),
            SearchMode::Files => self.filtered_files.len(),
        }
    }

    pub(crate) fn current_selection(&self) -> Option<SearchSelection> {
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
}
