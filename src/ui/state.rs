use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use ratatui::widgets::TableState;
use throbber_widgets_tui::ThrobberState;

use super::config::UiConfig;
use crate::systems::filesystem::IndexUpdate;
use crate::systems::search;
use frz_plugin_api::{
    PluginSelectionContext, SearchData, SearchMode, SearchPluginRegistry, SearchSelection,
};
use frz_tui::components::IndexProgress;
use frz_tui::input::SearchInput;
pub use frz_tui::theme::Theme;

mod search_runtime;

use search_runtime::SearchRuntime;

impl<'a> Drop for App<'a> {
    fn drop(&mut self) {
        self.search.shutdown();
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
    pub(crate) index_updates: Option<Receiver<IndexUpdate>>,
    pub(super) search: SearchRuntime,
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
        let mut plugins = SearchPluginRegistry::default();
        crate::plugins::builtin::register_builtin_plugins(&mut plugins);
        Self::with_plugins(data, plugins)
    }

    pub fn with_plugins(data: SearchData, plugins: SearchPluginRegistry) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let initial_query = data.initial_query.clone();
        let context_label = data.context_label.clone();
        let mut index_progress = IndexProgress::new();
        let worker_plugins = plugins.clone();
        let (search_tx, search_rx, search_latest_query_id) =
            search::spawn(data.clone(), worker_plugins);
        let search = SearchRuntime::new(search_tx, search_rx, search_latest_query_id);
        let mut tab_states = HashMap::new();
        for plugin in plugins.iter() {
            let mode = plugin.mode();
            tab_states.insert(mode, TabBuffers::default());
            index_progress.register_dataset(plugin.descriptor().dataset.key());
        }
        let mut ui = UiConfig::default();
        for descriptor in plugins.descriptors() {
            ui.register_plugin(descriptor);
        }
        let mode = plugins
            .iter()
            .next()
            .map(|plugin| plugin.mode())
            .or_else(|| ui.tabs().first().map(|tab| tab.mode))
            .unwrap_or_else(crate::plugins::builtin::facets::mode);

        index_progress.refresh_from_data(
            &data,
            plugins.iter().map(|plugin| {
                let dataset = plugin.dataset();
                (dataset.key(), dataset.total_count(&data))
            }),
        );

        Self {
            data,
            mode,
            search_input: SearchInput::new(initial_query),
            table_state,
            input_title: context_label,
            ui,
            theme: Theme::default(),
            throbber_state: ThrobberState::default(),
            index_progress,
            tab_states,
            plugins,
            index_updates: None,
            search,
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
        self.search.mark_query_dirty();
    }

    pub(crate) fn mark_query_dirty_from_user_input(&mut self) {
        self.search.mark_query_dirty_from_user_input();
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

    pub(crate) fn dataset_totals(&self) -> Vec<(&'static str, usize)> {
        self.plugins
            .iter()
            .map(|plugin| {
                let dataset = plugin.dataset();
                (dataset.key(), dataset.total_count(&self.data))
            })
            .collect()
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
    use frz_plugin_api::{FacetRow, FileRow};

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
        while app.search.is_in_flight() && Instant::now() < deadline {
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
            .get(&crate::plugins::builtin::facets::mode())
            .map(|state| !state.filtered.is_empty())
            .unwrap_or(false);
        let files_ready = app
            .tab_states
            .get(&crate::plugins::builtin::files::mode())
            .map(|state| !state.filtered.is_empty())
            .unwrap_or(false);
        assert!(
            facets_ready || files_ready,
            "expected initial search results to populate"
        );
    }
}
