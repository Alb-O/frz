use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{Receiver, Sender};

use ratatui::widgets::TableState;
use throbber_widgets_tui::ThrobberState;

use super::components::progress::IndexProgress;
use crate::input::SearchInput;
use crate::plugins::{
    PluginSelectionContext, SearchMode, SearchPluginRegistry, builtin::FACETS_MODE,
};
use crate::systems::filesystem::IndexUpdate;
use crate::systems::search::{self, SearchCommand, SearchResult};
use crate::theme::Theme;
use crate::types::{SearchData, SearchSelection, UiConfig};

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
    pub(crate) input_title: Option<String>,
    pub(crate) ui: UiConfig,
    pub theme: Theme,
    pub(crate) throbber_state: ThrobberState,
    pub(crate) index_progress: IndexProgress,
    pub(crate) tab_states: HashMap<SearchMode, TabBuffers>,
    plugins: SearchPluginRegistry,
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

#[derive(Default)]
pub(crate) struct TabBuffers {
    pub filtered: Vec<usize>,
    pub scores: Vec<u16>,
    pub headers: Option<Vec<String>>,
    pub widths: Option<Vec<ratatui::layout::Constraint>>,
}

impl<'a> App<'a> {
    pub fn new(data: SearchData) -> Self {
        Self::with_plugins(data, SearchPluginRegistry::default())
    }

    pub fn with_plugins(data: SearchData, plugins: SearchPluginRegistry) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let initial_query = data.initial_query.clone();
        let context_label = data.context_label.clone();
        let index_progress = IndexProgress::from_plugins(
            &data,
            plugins.definitions().map(|definition| definition.mode()),
        );
        let worker_plugins = plugins.clone();
        let (search_tx, search_rx, search_latest_query_id) =
            search::spawn(data.clone(), worker_plugins);
        let mut tab_states = HashMap::new();
        for definition in plugins.definitions() {
            tab_states.insert(definition.mode(), TabBuffers::default());
        }
        let mode = plugins
            .definitions()
            .next()
            .map(|definition| definition.mode())
            .unwrap_or(FACETS_MODE);

        Self {
            data,
            mode,
            search_input: SearchInput::new(initial_query),
            table_state,
            input_title: context_label,
            ui: UiConfig::for_definitions(plugins.definitions()),
            theme: Theme::default(),
            throbber_state: ThrobberState::default(),
            index_progress,
            tab_states,
            plugins,
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
            self.ensure_tab_buffers();
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
        self.tab_states
            .get(&self.mode)
            .map(|state| state.filtered.len())
            .unwrap_or(0)
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
        let state = self.tab_states.get(&self.mode)?;
        let index = *state.filtered.get(selected)?;
        let plugin = self.plugins.plugin(self.mode)?;
        let context = PluginSelectionContext::new(&self.data);
        plugin.selection(context, index)
    }

    pub(crate) fn ensure_tab_buffers(&mut self) {
        for plugin in self.plugins.iter() {
            self.tab_states.entry(plugin.mode()).or_default();
        }
        for tab in self.ui.tabs() {
            self.tab_states.entry(tab.mode).or_default();
        }
    }

    pub(crate) fn plugin_modes(&self) -> Vec<SearchMode> {
        self.plugins.definitions().map(|definition| definition.mode()).collect()
    }

    pub(crate) fn plugin_definition(
        &self,
        mode: SearchMode,
    ) -> Option<&'static crate::plugins::SearchPluginDefinition> {
        self.plugins.definition(mode)
    }

    pub fn set_headers_for(&mut self, mode: SearchMode, headers: Vec<String>) {
        self.tab_states.entry(mode).or_default().headers = Some(headers);
    }

    pub fn set_widths_for(&mut self, mode: SearchMode, widths: Vec<ratatui::layout::Constraint>) {
        self.tab_states.entry(mode).or_default().widths = Some(widths);
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::plugins::builtin::FILES_MODE;
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
        let facets_ready = app
            .tab_states
            .get(&FACETS_MODE)
            .map(|state| !state.filtered.is_empty())
            .unwrap_or(false);
        let files_ready = app
            .tab_states
            .get(&FILES_MODE)
            .map(|state| !state.filtered.is_empty())
            .unwrap_or(false);
        assert!(
            facets_ready || files_ready,
            "expected initial search results to populate"
        );
    }
}
