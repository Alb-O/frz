use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{Receiver, Sender};

use ratatui::widgets::TableState;
use throbber_widgets_tui::ThrobberState;

use super::components::progress::IndexProgress;
use crate::indexing::IndexUpdate;
use crate::input::SearchInput;
use crate::search::{self, SearchCommand, SearchResult};
use crate::theme::Theme;
use crate::types::{SearchData, SearchMode, SearchSelection, UiConfig};

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
    #[cfg_attr(not(feature = "fs"), allow(dead_code))]
    pub(crate) index_updates: Option<Receiver<IndexUpdate>>,
    pub(super) search_tx: Sender<SearchCommand>,
    pub(super) search_rx: Receiver<SearchResult>,
    pub(super) search_latest_query_id: Arc<AtomicU64>,
    pub(super) next_query_id: u64,
    pub(super) latest_query_id: Option<u64>,
    pub(super) search_in_flight: bool,
    pub(super) input_revision: u64,
    pub(super) pending_result_revision: u64,
    pub(super) last_applied_revision: u64,
    pub(super) last_user_input_revision: u64,
}

impl<'a> App<'a> {
    pub fn new(data: SearchData) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let initial_query = data.initial_query.clone();
        let context_label = data.context_label.clone();
        let index_progress = IndexProgress::from(&data);
        let (search_tx, search_rx, search_latest_query_id) = search::spawn(data.clone());

        Self {
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
            search_in_flight: false,
            input_revision: 0,
            pending_result_revision: 0,
            last_applied_revision: 0,
            last_user_input_revision: 0,
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_mode(&mut self, mode: SearchMode) {
        if self.mode != mode {
            self.mode = mode;
            self.table_state.select(Some(0));
            self.mark_query_dirty();
            self.request_search();
        }
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

    pub(crate) fn mark_query_dirty(&mut self) {
        self.input_revision = self.input_revision.wrapping_add(1);
    }

    pub(crate) fn mark_query_dirty_from_user_input(&mut self) {
        self.mark_query_dirty();
        self.last_user_input_revision = self.input_revision;
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::types::{FacetRow, FileRow};

    fn sample_data() -> SearchData {
        let mut data = SearchData::new();
        data.facets = vec![
            FacetRow::new("alpha", 3),
            FacetRow::new("beta", 5),
            FacetRow::new("gamma", 2),
        ];
        data.files = vec![
            FileRow::new("src/main.rs", ["alpha", "beta"]),
            FileRow::new("src/lib.rs", ["beta"]),
            FileRow::new("README.md", ["gamma"]),
        ];
        data
    }

    fn prime_and_wait_for_results(app: &mut App) {
        app.mark_query_dirty();
        app.request_search();

        let deadline = Instant::now() + Duration::from_secs(1);
        while app.search_in_flight && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
            app.pump_search_results();
        }
        app.pump_search_results();
    }

    #[test]
    fn new_app_hydrates_initial_results() {
        let data = sample_data();
        let mut app = App::new(data);
        prime_and_wait_for_results(&mut app);
        assert!(
            !app.filtered_facets.is_empty() || !app.filtered_files.is_empty(),
            "expected initial search results to populate"
        );
    }
}
