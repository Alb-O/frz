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
use crate::extensions::api::{FILES_DATASET_KEY, SearchData, SearchSelection};
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
	pub search_input: SearchInput<'a>,
	pub table_state: TableState,
	pub(crate) input_title: Option<String>,
	pub(crate) ui: UiConfig,
	pub theme: Theme,
	pub(crate) bat_theme: Option<String>,
	pub(crate) throbber_state: ThrobberState,
	pub(crate) index_progress: IndexProgress,
	pub(crate) tab_buffers: TabBuffers,
	pub(crate) row_id_map: HashMap<u64, usize>,
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
		let mut table_state = TableState::default();
		table_state.select(Some(0));
		let initial_query = data.initial_query.clone();
		let context_label = data.context_label.clone();
		let mut index_progress = IndexProgress::new();
		let (search_tx, search_rx, search_latest_query_id) = search::spawn(data.clone());
		let search = SearchRuntime::new(search_tx, search_rx, search_latest_query_id);

		index_progress.register_dataset(FILES_DATASET_KEY);
		let ui = UiConfig::default();

		index_progress.refresh_from_data(&data, [(FILES_DATASET_KEY, data.files.len())]);

		let row_id_map = data
			.id_map_for_dataset(FILES_DATASET_KEY)
			.unwrap_or_default();

		Self {
			data,
			search_input: SearchInput::new(initial_query),
			table_state,
			input_title: context_label,
			ui,
			theme: Theme::default(),
			bat_theme: None,
			throbber_state: ThrobberState::default(),
			index_progress,
			tab_buffers: TabBuffers::default(),
			row_id_map,
			index_updates: None,
			search,
			initial_results_deadline: None,
			initial_results_timeout: Some(Duration::from_millis(250)),
		}
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
		self.tab_buffers.filtered.len()
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
		let index = *self.tab_buffers.filtered.get(selected)?;
		self.data
			.files
			.get(index)
			.cloned()
			.map(SearchSelection::File)
	}

	/// Ensure that every known search mode has backing buffers.
	pub(crate) fn ensure_tab_buffers(&mut self) {
		// No-op now that we have a single tab buffer
	}

	/// Rebuild the stable-id lookup tables from the current dataset.
	pub(crate) fn rebuild_row_id_maps(&mut self) {
		self.row_id_map = self
			.data
			.id_map_for_dataset(FILES_DATASET_KEY)
			.unwrap_or_default();
	}

	/// Apply a batch of matches, reconciling stable ids with indices when
	/// available.
	pub(crate) fn apply_match_batch(
		&mut self,
		indices: Vec<usize>,
		ids: Option<Vec<u64>>,
		scores: Vec<u16>,
	) {
		let filtered = if let Some(ids) = ids {
			let ids_len = ids.len();
			let mut resolved: Vec<usize> = ids
				.into_iter()
				.enumerate()
				.filter_map(|(offset, id)| {
					self.row_id_map
						.get(&id)
						.copied()
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
		self.tab_buffers.filtered = filtered;
		self.tab_buffers.scores = scores;
		let has_results = !self.tab_buffers.filtered.is_empty();
		self.settle_initial_results(has_results);
		self.ensure_selection();
	}

	/// Gather total counts for each registered dataset.
	pub(crate) fn dataset_totals(&self) -> Vec<(&'static str, usize)> {
		vec![(FILES_DATASET_KEY, self.data.files.len())]
	}

	/// Update headers for the file search.
	pub fn set_headers(&mut self, headers: Vec<String>) {
		self.tab_buffers.headers = Some(headers);
	}

	/// Update column widths for the file search.
	pub fn set_widths(&mut self, widths: Vec<ratatui::layout::Constraint>) {
		self.tab_buffers.widths = Some(widths);
	}
}

#[cfg(test)]
mod tests {
	use std::time::{Duration, Instant};

	use super::*;
	use crate::extensions::api::{FileRow, MatchBatch, SearchViewV2};

	fn sample_data() -> SearchData {
		let mut data = SearchData::new();
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
		let files_ready = !app.tab_buffers.filtered.is_empty();
		assert!(files_ready, "expected initial search results to populate");
	}

	#[test]
	fn stable_ids_survive_reordering() {
		let mut data = sample_data();
		let original_id = data.files[1].id.expect("expected stable id");

		let mut app = App::new(data.clone());
		assert_eq!(
			app.row_id_map.get(&original_id).copied(),
			Some(1),
			"id map should point at the original index"
		);

		let moved = data.files.remove(1);
		data.files.insert(0, moved);
		app.data = data;
		app.rebuild_row_id_maps();
		assert_eq!(
			app.row_id_map.get(&original_id).copied(),
			Some(0),
			"id map should reflect reordered rows"
		);

		let batch = MatchBatch {
			indices: vec![1],
			ids: Some(vec![original_id]),
			scores: vec![42],
		};
		app.replace_matches_v2(batch);
		assert_eq!(app.tab_buffers.filtered, vec![0]);
	}
}
