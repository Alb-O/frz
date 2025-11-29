//! Core state container for the terminal application's front-end.
//!
//! The `app` module exposes the [`App`] struct which bundles together search
//! data, extension metadata, and UI-specific caches.

use std::sync::mpsc::Receiver;

use frz_core::filesystem::indexer::IndexResult;
use frz_core::filesystem::search::{
	FILES_DATASET_KEY, SearchData, SearchSelection, runtime as search,
};
use throbber_widgets_tui::ThrobberState;

use super::SearchRuntime;
use super::preview::PreviewState;
use super::results::ResultsState;
use crate::components::{IndexProgress, PreviewContent, PreviewKind, wrap_highlighted_lines};
use crate::config::UiLabels;
use crate::input::QueryInput;
use crate::style::{StyleConfig, Theme};

impl<'a> Drop for App<'a> {
	fn drop(&mut self) {
		self.search.shutdown();
	}
}

/// Aggregate state shared across the terminal UI.
///
/// The `App` owns the current search data, manages extension-defined
/// contributions, and keeps track of UI affordances.
pub struct App<'a> {
	/// Current search data including files and metadata.
	pub data: SearchData,
	/// Text input widget for the search filter.
	pub search_input: QueryInput<'a>,
	pub(crate) ui: UiLabels,
	/// Current style and theme configuration.
	pub style: StyleConfig,
	pub(crate) bat_theme: Option<String>,
	pub(crate) throbber_state: ThrobberState,
	pub(crate) index_progress: IndexProgress,
	pub(crate) index_updates: Option<Receiver<IndexResult>>,
	pub(crate) search: SearchRuntime,
	pub(crate) preview: PreviewState,
	pub(crate) results: ResultsState,
}

impl<'a> App<'a> {
	/// Construct an [`App`] with the builtin extension catalog.
	pub fn new(data: SearchData) -> Self {
		let initial_query = data.initial_query.clone();
		let results = Self::init_results(&data);
		let preview = Self::init_preview();
		let (search, index_progress) = Self::init_search_and_indexing(&data);

		Self {
			data,
			search_input: QueryInput::new(initial_query),
			ui: UiLabels::default(),
			style: StyleConfig::default(),
			bat_theme: None,
			throbber_state: ThrobberState::default(),
			index_progress,
			index_updates: None,
			search,
			preview,
			results,
		}
	}

	fn init_results(data: &SearchData) -> ResultsState {
		let row_id_map = data
			.id_map_for_dataset(FILES_DATASET_KEY)
			.unwrap_or_default();

		ResultsState {
			row_id_map,
			..ResultsState::default()
		}
	}

	fn init_preview() -> PreviewState {
		let mut preview = PreviewState::new();
		preview.viewport_height = 1;
		preview.wrap_width = 80;
		preview
	}

	fn init_search_and_indexing(data: &SearchData) -> (SearchRuntime, IndexProgress) {
		let (search_tx, search_rx, search_latest_query_id) = search::spawn(data.clone());
		let search = SearchRuntime::new(search_tx, search_rx, search_latest_query_id);

		let mut index_progress = IndexProgress::new();
		index_progress.register_dataset(FILES_DATASET_KEY);
		index_progress.refresh_from_data(data, [(FILES_DATASET_KEY, data.files.len())]);

		(search, index_progress)
	}

	/// Apply a new theme without changing the associated bat configuration.
	pub fn set_theme(&mut self, theme: Theme) {
		self.set_theme_with_bat(theme, None);
	}

	/// Apply a new theme and optional bat theme name.
	pub fn set_theme_with_bat(&mut self, theme: Theme, bat_theme: Option<String>) {
		self.style.theme = theme;
		self.bat_theme = bat_theme;
	}

	/// Ensure the row selection remains valid for the currently filtered list.
	pub(crate) fn ensure_selection(&mut self) {
		self.results.ensure_selection();
	}

	/// Return the number of filtered entries for the active tab.
	pub(crate) fn filtered_len(&self) -> usize {
		self.results.filtered_len()
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
		let selected = self.results.table_state.selected()?;
		let index = *self.results.buffers.filtered.get(selected)?;
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
		self.results.row_id_map = self
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
		// Track the path of the currently selected item before updating results
		let old_selected_path = if self.preview.enabled {
			self.current_selection().map(|sel| match sel {
				SearchSelection::File(file) => {
					self.data.resolve_file_path(&file).display().to_string()
				}
			})
		} else {
			None
		};

		let filtered = if let Some(ids) = ids {
			let ids_len = ids.len();
			let mut resolved: Vec<usize> = ids
				.into_iter()
				.enumerate()
				.filter_map(|(offset, id)| {
					self.results
						.row_id_map
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
		self.results.buffers.filtered = filtered;
		self.results.buffers.scores = scores;
		self.ensure_selection();

		// Update preview if enabled and the selected item changed
		if self.preview.enabled {
			let new_selected_path = self.current_selection().map(|sel| match sel {
				SearchSelection::File(file) => {
					self.data.resolve_file_path(&file).display().to_string()
				}
			});

			// Trigger preview update if the selected item changed (or if we didn't have one before)
			if old_selected_path != new_selected_path {
				self.update_preview();
			}
		}
	}

	/// Gather total counts for each registered dataset.
	pub(crate) fn dataset_totals(&self) -> Vec<(&'static str, usize)> {
		vec![(FILES_DATASET_KEY, self.data.files.len())]
	}

	/// Update headers for the file search.
	pub fn set_headers(&mut self, headers: Vec<String>) {
		self.results.buffers.headers = Some(headers);
	}

	/// Update column widths for the file search.
	pub fn set_widths(&mut self, widths: Vec<ratatui::layout::Constraint>) {
		self.results.buffers.widths = Some(widths);
	}

	/// Toggle the preview pane visibility.
	pub(crate) fn toggle_preview(&mut self) {
		self.preview.enabled = !self.preview.enabled;
		if self.preview.enabled {
			self.update_preview();
		}
	}

	/// Enable the preview pane.
	pub fn enable_preview(&mut self) {
		self.preview.enabled = true;
		self.update_preview();
	}

	/// Disable the preview pane.
	pub fn disable_preview(&mut self) {
		self.preview.enabled = false;
		self.preview.area = None;
		self.preview.scrollbar_area = None;
		self.preview.hovered = false;
		self.results.hovered = false;
		self.results.dragging = false;
		self.preview.dragging = false;
		self.preview.wrapped_lines.clear();
	}

	/// Update preview visibility based on terminal width.
	pub(crate) fn update_preview_responsive(&mut self, width: u16) {
		const MIN_WIDTH_FOR_PREVIEW: u16 = 100;

		let should_enable = width >= MIN_WIDTH_FOR_PREVIEW;

		if should_enable && !self.preview.enabled {
			self.enable_preview();
		} else if !should_enable && self.preview.enabled {
			self.disable_preview();
		}
	}

	/// Update the preview content for the currently selected file.
	/// The previous preview remains visible until the new one is ready.
	pub(crate) fn update_preview(&mut self) {
		if !self.preview.enabled {
			return;
		}

		let selection = match self.current_selection() {
			Some(SearchSelection::File(file)) => file,
			_ => {
				self.preview.content = PreviewContent::empty();
				self.preview.path.clear();
				self.preview.pending_path = None;
				self.preview.scroll = 0;
				return;
			}
		};

		let path = self.data.resolve_file_path(&selection);
		let path_str = path.display().to_string();

		// Skip if we already have this preview cached or it's already pending
		if self.preview.path == path_str {
			self.preview.pending_path = None;
			return;
		}
		if self.preview.pending_path.as_ref() == Some(&path_str) {
			return;
		}

		// Mark that we're loading this path, but keep displaying the old preview
		self.preview.pending_path = Some(path_str);

		// Request preview generation in background
		self.preview
			.runtime
			.request(path, self.bat_theme.clone(), 500);
	}

	/// Poll for completed preview results from the background worker.
	pub(crate) fn pump_preview_results(&mut self) {
		use std::sync::mpsc::TryRecvError;

		loop {
			match self.preview.runtime.try_recv() {
				Ok(result) => {
					// Only apply if this is still the current request
					if self.preview.runtime.is_current(result.id) {
						// Update the displayed preview and clear pending state
						self.preview.path = result.content.path.clone();
						self.preview.content = result.content;
						self.preview.pending_path = None;
						self.preview.scroll = 0;
						self.rebuild_preview_wrap(self.preview.wrap_width);
						self.preview.update_scrollbar();
					}
				}
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => break,
			}
		}
	}

	/// Scroll the preview pane up.
	pub(crate) fn scroll_preview_up(&mut self, lines: usize) {
		self.preview.scroll_up(lines);
	}

	/// Scroll the preview pane down.
	pub(crate) fn scroll_preview_down(&mut self, lines: usize) {
		self.preview.scroll_down(lines);
	}

	pub(crate) fn update_preview_hover(&mut self, column: u16, row: u16) {
		self.preview.update_hover(column, row);
	}

	pub(crate) fn update_results_hover(&mut self, column: u16, row: u16) {
		self.results.update_hover(column, row);
	}

	pub(crate) fn select_result_at(&mut self, column: u16, row: u16) -> bool {
		self.results.select_at(column, row)
	}

	/// Update scrollbar state to match current preview content and scroll position.
	pub(crate) fn update_scrollbar_state(&mut self) {
		self.preview.update_scrollbar();
	}

	pub(crate) fn rebuild_preview_wrap(&mut self, available_width: usize) {
		self.preview.wrap_width = available_width;

		self.preview.wrapped_lines = match &self.preview.content.kind {
			PreviewKind::Text { lines } => wrap_highlighted_lines(lines, available_width),
			_ => Vec::new(),
		};

		let content_length = self.preview.wrapped_lines.len();
		let max_scroll = self.preview.max_scroll(content_length);
		self.preview.scroll = self.preview.scroll.min(max_scroll);
	}
}

#[cfg(test)]
mod tests {
	use std::time::{Duration, Instant};

	use frz_core::filesystem::search::{FileRow, MatchBatch, SearchViewV2};
	use ratatui::layout::Rect;
	use ratatui::text::Line;

	use super::*;

	fn sample_data() -> SearchData {
		let mut data = SearchData::new();
		data.files = vec![
			FileRow::new("src/main.rs"),
			FileRow::new("src/lib.rs"),
			FileRow::new("README.md"),
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
		let files_ready = !app.results.buffers.filtered.is_empty();
		assert!(files_ready, "expected initial search results to populate");
	}

	#[test]
	fn stable_ids_survive_reordering() {
		let mut data = sample_data();
		let original_id = data.files[1].id.expect("expected stable id");

		let mut app = App::new(data.clone());
		assert_eq!(
			app.results.row_id_map.get(&original_id).copied(),
			Some(1),
			"id map should point at the original index"
		);

		let moved = data.files.remove(1);
		data.files.insert(0, moved);
		app.data = data;
		app.rebuild_row_id_maps();
		assert_eq!(
			app.results.row_id_map.get(&original_id).copied(),
			Some(0),
			"id map should reflect reordered rows"
		);

		let batch = MatchBatch {
			indices: vec![1],
			ids: Some(vec![original_id]),
			scores: vec![42],
		};
		app.replace_matches_v2(batch);
		assert_eq!(app.results.buffers.filtered, vec![0]);
	}

	#[test]
	fn dragging_scrollbar_respects_wrapped_lines() {
		let mut app = App::new(sample_data());
		app.preview.enabled = true;
		app.preview.viewport_height = 8;
		app.preview.wrapped_lines = vec![Line::from("x"); 30];
		app.preview.content = PreviewContent::text("wrapped.rs", vec![Line::from("x"); 5]);
		app.preview.scrollbar_area = Some(Rect::new(0, 0, 1, 8));
		app.update_scrollbar_state();

		let area = app.preview.scrollbar_area.unwrap();
		let bottom = area.y.saturating_add(area.height).saturating_sub(1);
		let dragged = app.drag_preview_scrollbar_to(bottom);
		assert!(dragged, "drag should succeed when scrollbar is present");

		let expected_max = app.preview.max_scroll(app.preview.wrapped_lines.len());
		assert_eq!(
			app.preview.scroll, expected_max,
			"dragging to the bottom should reach max scroll based on wrapped lines"
		);
	}
}
