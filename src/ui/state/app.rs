//! Core state container for the terminal application's front-end.
//!
//! The `app` module exposes the [`App`] struct which bundles together search
//! data, extension metadata, and UI-specific caches.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use ratatui::widgets::TableState;
use throbber_widgets_tui::ThrobberState;

use super::SearchRuntime;
use crate::extensions::api::{
    ContributionStores, ExtensionCatalog, ExtensionSelectionContext, SearchData, SearchMode,
    SearchSelection,
};
use crate::systems::filesystem::IndexResult;
use crate::systems::search;
use crate::tui::components::IndexProgress;
use crate::tui::input::SearchInput;
pub use crate::tui::theme::Theme;
use crate::ui::config::UiConfig;

impl<'a> Drop for App<'a> {
    fn drop(&mut self) {
        self.search.shutdown();
    }
}

/// Aggregate state shared across the terminal UI.
///
/// The `App` owns the current search data, manages extension-defined
/// contributions, and keeps track of UI affordances such as tab buffers and
/// loading indicators.  Splitting the implementation into smaller modules lets
/// call-sites interact with a focused surface area while the underlying state
/// remains centralized here.
pub struct App<'a> {
    pub data: SearchData,
    pub mode: SearchMode,
    pub search_input: SearchInput<'a>,
    pub table_state: TableState,
    pub(crate) input_title: Option<String>,
    pub(crate) ui: UiConfig,
    pub theme: Theme,
    pub(crate) bat_theme: Option<String>,
    pub(crate) git_modifications: bool,
    pub(crate) throbber_state: ThrobberState,
    pub(crate) index_progress: IndexProgress,
    pub(crate) tab_states: HashMap<SearchMode, TabBuffers>,
    pub(crate) row_id_maps: HashMap<SearchMode, HashMap<u64, usize>>,
    extensions: ExtensionCatalog,
    pub(crate) index_updates: Option<Receiver<IndexResult>>,
    pub(in crate::ui) search: SearchRuntime,
    pub(crate) initial_results_deadline: Option<Instant>,
    pub(crate) initial_results_timeout: Option<Duration>,
}

/// Cache of rendered rows for a specific tab.
#[derive(Default)]
pub(crate) struct TabBuffers {
    pub filtered: Vec<usize>,
    pub scores: Vec<u16>,
    pub headers: Option<Vec<String>>,
    pub widths: Option<Vec<ratatui::layout::Constraint>>,
}

impl<'a> App<'a> {
    /// Construct an [`App`] with the builtin extension catalog.
    pub fn new(data: SearchData) -> Self {
        crate::logging::initialize();
        let mut extensions = ExtensionCatalog::default();
        crate::extensions::builtin::register_builtin_extensions(&mut extensions)
            .expect("builtin extensions must register successfully");
        Self::with_extensions(data, extensions)
    }

    /// Construct an [`App`] using the provided extension catalog.
    pub fn with_extensions(data: SearchData, extensions: ExtensionCatalog) -> Self {
        crate::logging::initialize();
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        let initial_query = data.initial_query.clone();
        let context_label = data.context_label.clone();
        let mut index_progress = IndexProgress::new();
        let worker_extensions = extensions.clone();
        let (search_tx, search_rx, search_latest_query_id) =
            search::spawn(data.clone(), worker_extensions);
        let search = SearchRuntime::new(search_tx, search_rx, search_latest_query_id);
        let mut tab_states = HashMap::new();
        for module in extensions.modules() {
            let mode = module.mode();
            tab_states.insert(mode, TabBuffers::default());
            index_progress.register_dataset(module.descriptor().dataset.key());
        }
        let mut ui = UiConfig::default();
        for descriptor in extensions.descriptors() {
            ui.register_extension(descriptor);
        }
        let mode = extensions
            .modules()
            .next()
            .map(|module| module.mode())
            .or_else(|| ui.tabs().first().map(|tab| tab.mode))
            .unwrap_or_else(crate::extensions::builtin::attributes::mode);

        index_progress.refresh_from_data(
            &data,
            extensions.modules().map(|module| {
                let dataset = module.dataset();
                (dataset.key(), dataset.total_count(&data))
            }),
        );

        let mut app = Self {
            data,
            mode,
            search_input: SearchInput::new(initial_query),
            table_state,
            input_title: context_label,
            ui,
            theme: Theme::default(),
            bat_theme: None,
            git_modifications: true,
            throbber_state: ThrobberState::default(),
            index_progress,
            tab_states,
            row_id_maps: HashMap::new(),
            extensions,
            index_updates: None,
            search,
            initial_results_deadline: None,
            initial_results_timeout: Some(Duration::from_millis(250)),
        };
        app.rebuild_row_id_maps();
        app
    }

    /// Apply a new theme without changing the associated bat configuration.
    pub fn set_theme(&mut self, theme: Theme) {
        self.set_theme_with_bat(theme, None);
    }

    /// Apply a new theme and optional bat theme name.
    pub fn set_theme_with_bat(&mut self, theme: Theme, bat_theme: Option<String>) {
        self.theme = theme;
        self.bat_theme = bat_theme;
    }

    /// Enable or disable Git modification highlights in previews.
    pub fn set_git_modifications(&mut self, enabled: bool) {
        self.git_modifications = enabled;
    }

    /// Switch to a different search mode and reset the selection.
    pub fn set_mode(&mut self, mode: SearchMode) {
        if self.mode != mode {
            self.mode = mode;
            self.table_state.select(Some(0));
            self.ensure_tab_buffers();
            self.mark_query_dirty();
            self.request_search();
        }
    }

    /// Ensure the row selection remains valid for the currently filtered list.
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

    /// Return the number of filtered entries for the active tab.
    pub(crate) fn filtered_len(&self) -> usize {
        self.tab_states
            .get(&self.mode)
            .map(|state| state.filtered.len())
            .unwrap_or(0)
    }

    /// Flag the in-memory query so the next search run refreshes it.
    pub(crate) fn mark_query_dirty(&mut self) {
        self.search.mark_query_dirty();
    }

    /// Flag the query after direct user input, ensuring freshness even if
    /// indexing updates are pending.
    pub(crate) fn mark_query_dirty_from_user_input(&mut self) {
        self.search.mark_query_dirty_from_user_input();
    }

    /// Compute the currently selected row using extension-specific logic.
    pub(crate) fn current_selection(&self) -> Option<SearchSelection> {
        let selected = self.table_state.selected()?;
        let state = self.tab_states.get(&self.mode)?;
        let index = *state.filtered.get(selected)?;
        let module = self.extensions.module(self.mode)?;
        let context = ExtensionSelectionContext::new(&self.data);
        module.selection(context, index)
    }

    /// Ensure that every known search mode has backing buffers.
    pub(crate) fn ensure_tab_buffers(&mut self) {
        for module in self.extensions.modules() {
            self.tab_states.entry(module.mode()).or_default();
            self.row_id_maps.entry(module.mode()).or_default();
        }
        for tab in self.ui.tabs() {
            self.tab_states.entry(tab.mode).or_default();
            self.row_id_maps.entry(tab.mode).or_default();
        }
    }

    /// Rebuild the stable-id lookup tables from the current dataset.
    pub(crate) fn rebuild_row_id_maps(&mut self) {
        self.row_id_maps.clear();
        for module in self.extensions.modules() {
            let mode = module.mode();
            if let Some(map) = self
                .data
                .id_map_for_dataset(module.descriptor().dataset.key())
            {
                self.row_id_maps.insert(mode, map);
            }
        }
    }

    /// Apply a batch of matches, reconciling stable ids with indices when
    /// available.
    pub(crate) fn apply_match_batch(
        &mut self,
        mode: SearchMode,
        indices: Vec<usize>,
        ids: Option<Vec<u64>>,
        scores: Vec<u16>,
    ) {
        self.ensure_tab_buffers();
        let entry = self.tab_states.entry(mode).or_default();
        let filtered = if let Some(ids) = ids {
            let ids_len = ids.len();
            let map = self.row_id_maps.get(&mode);
            let mut resolved: Vec<usize> = ids
                .into_iter()
                .enumerate()
                .filter_map(|(offset, id)| {
                    map.and_then(|lookup| lookup.get(&id).copied())
                        .or_else(|| indices.get(offset).copied())
                })
                .collect();
            if ids_len < indices.len() {
                resolved.extend(indices.into_iter().skip(ids_len));
            }
            resolved
        } else {
            indices
        };
        entry.filtered = filtered;
        entry.scores = scores;
        let has_results = !entry.filtered.is_empty();
        self.settle_initial_results(has_results);
        self.ensure_selection();
    }

    /// Gather total counts for each registered dataset.
    pub(crate) fn dataset_totals(&self) -> Vec<(&'static str, usize)> {
        self.extensions
            .modules()
            .map(|module| {
                let dataset = module.dataset();
                (dataset.key(), dataset.total_count(&self.data))
            })
            .collect()
    }

    /// Update headers for a specific search mode.
    pub fn set_headers_for(&mut self, mode: SearchMode, headers: Vec<String>) {
        self.tab_states.entry(mode).or_default().headers = Some(headers);
    }

    /// Update column widths for a specific search mode.
    pub fn set_widths_for(&mut self, mode: SearchMode, widths: Vec<ratatui::layout::Constraint>) {
        self.tab_states.entry(mode).or_default().widths = Some(widths);
    }

    /// Return the currently registered contribution stores.
    pub(crate) fn contributions(&self) -> ContributionStores {
        self.extensions.contributions()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::extensions::api::{AttributeRow, FileRow, MatchBatch, SearchViewV2};

    fn sample_data() -> SearchData {
        let mut data = SearchData::new();
        data.attributes = vec![
            AttributeRow::new("alpha", 3),
            AttributeRow::new("beta", 5),
            AttributeRow::new("gamma", 2),
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
        let attributes_ready = app
            .tab_states
            .get(&crate::extensions::builtin::attributes::mode())
            .map(|state| !state.filtered.is_empty())
            .unwrap_or(false);
        let files_ready = app
            .tab_states
            .get(&crate::extensions::builtin::files::mode())
            .map(|state| !state.filtered.is_empty())
            .unwrap_or(false);
        assert!(
            attributes_ready || files_ready,
            "expected initial search results to populate"
        );
    }

    #[test]
    fn stable_ids_survive_reordering() {
        let mut data = sample_data();
        let files_mode = crate::extensions::builtin::files::mode();
        let original_id = data.files[1].id.expect("expected stable id");

        let mut app = App::new(data.clone());
        assert_eq!(
            app.row_id_maps
                .get(&files_mode)
                .and_then(|map| map.get(&original_id))
                .copied(),
            Some(1),
            "id map should point at the original index"
        );

        let moved = data.files.remove(1);
        data.files.insert(0, moved);
        app.data = data;
        app.rebuild_row_id_maps();
        assert_eq!(
            app.row_id_maps
                .get(&files_mode)
                .and_then(|map| map.get(&original_id))
                .copied(),
            Some(0),
            "id map should reflect reordered rows"
        );

        let batch = MatchBatch {
            indices: vec![1],
            ids: Some(vec![original_id]),
            scores: vec![42],
        };
        app.replace_matches_v2(files_mode, batch);
        let state = app.tab_states.get(&files_mode).expect("tab state");
        assert_eq!(state.filtered, vec![0]);
    }
}
