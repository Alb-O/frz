//! Core state container for the terminal application's front-end.
//!
//! The `app` module exposes the [`App`] struct which bundles together search
//! data, extension metadata, and UI-specific caches.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use frz_core::filesystem_indexer::IndexResult;
use frz_core::search_pipeline::{
	FILES_DATASET_KEY, SearchData, SearchSelection, runtime as search,
};
use ratatui::layout::Rect;
use ratatui::widgets::{ScrollbarState, TableState};
use throbber_widgets_tui::ThrobberState;

use super::SearchRuntime;
use crate::components::{IndexProgress, PreviewContent, PreviewRuntime};
use crate::config::UiConfig;
use crate::input::SearchInput;
use crate::style::{StyleConfig, Theme};

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
	/// Current search data including files and metadata.
	pub data: SearchData,
	/// Text input widget for the search filter.
	pub search_input: SearchInput<'a>,
	/// Selection state for the results table.
	pub table_state: TableState,
	pub(crate) ui: UiConfig,
	/// Current style and theme configuration.
	pub style: StyleConfig,
	pub(crate) bat_theme: Option<String>,
	pub(crate) throbber_state: ThrobberState,
	pub(crate) index_progress: IndexProgress,
	pub(crate) tab_buffers: TabBuffers,
	pub(crate) row_id_map: HashMap<u64, usize>,
	pub(crate) index_updates: Option<Receiver<IndexResult>>,
	pub(crate) search: SearchRuntime,
	/// Whether the preview pane is visible.
	pub(crate) preview_enabled: bool,
	/// Cached preview content for the currently selected file.
	pub(crate) preview_content: PreviewContent,
	/// Scroll offset within the preview pane.
	pub(crate) preview_scroll: usize,
	/// Scrollbar state for the preview pane.
	pub(crate) preview_scrollbar_state: ScrollbarState,
	/// Last known viewport height for scroll bounds.
	pub(crate) preview_viewport_height: usize,
	/// Last known preview area on screen.
	pub(crate) preview_area: Option<Rect>,
	/// Whether the mouse is currently hovering the preview.
	pub(crate) preview_hovered: bool,
	/// Last known results area on screen.
	pub(crate) results_area: Option<Rect>,
	/// Whether the mouse is currently hovering the results table.
	pub(crate) results_hovered: bool,
	/// Whether the user is dragging within the results table.
	pub(crate) results_dragging: bool,
	/// Path of the file whose preview is currently displayed.
	pub(crate) preview_path: String,
	/// Path of the file we're currently loading a preview for (if any).
	pub(crate) pending_preview_path: Option<String>,
	/// Background preview generation runtime.
	pub(crate) preview_runtime: PreviewRuntime,
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
		let mut table_state = TableState::default();
		table_state.select(Some(0));
		let initial_query = data.initial_query.clone();
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
			ui,
			style: StyleConfig::default(),
			bat_theme: None,
			throbber_state: ThrobberState::default(),
			index_progress,
			tab_buffers: TabBuffers::default(),
			row_id_map,
			index_updates: None,
			search,
			preview_enabled: false,
			preview_content: PreviewContent::empty(),
			preview_scroll: 0,
			preview_scrollbar_state: ScrollbarState::default(),
			preview_viewport_height: 1,
			preview_area: None,
			preview_hovered: false,
			results_area: None,
			results_hovered: false,
			results_dragging: false,
			preview_path: String::new(),
			pending_preview_path: None,
			preview_runtime: PreviewRuntime::new(),
		}
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
		// Track the path of the currently selected item before updating results
		let old_selected_path = if self.preview_enabled {
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
		self.ensure_selection();

		// Update preview if enabled and the selected item changed
		if self.preview_enabled {
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
		self.tab_buffers.headers = Some(headers);
	}

	/// Update column widths for the file search.
	pub fn set_widths(&mut self, widths: Vec<ratatui::layout::Constraint>) {
		self.tab_buffers.widths = Some(widths);
	}

	/// Toggle the preview pane visibility.
	pub(crate) fn toggle_preview(&mut self) {
		self.preview_enabled = !self.preview_enabled;
		if self.preview_enabled {
			self.update_preview();
		}
	}

	/// Enable the preview pane.
	pub fn enable_preview(&mut self) {
		self.preview_enabled = true;
		self.update_preview();
	}

	/// Disable the preview pane.
	pub fn disable_preview(&mut self) {
		self.preview_enabled = false;
		self.preview_area = None;
		self.preview_hovered = false;
		self.results_hovered = false;
		self.results_dragging = false;
	}

	/// Update preview visibility based on terminal width.
	pub(crate) fn update_preview_responsive(&mut self, width: u16) {
		const MIN_WIDTH_FOR_PREVIEW: u16 = 100;

		let should_enable = width >= MIN_WIDTH_FOR_PREVIEW;

		if should_enable && !self.preview_enabled {
			self.enable_preview();
		} else if !should_enable && self.preview_enabled {
			self.disable_preview();
		}
	}

	/// Update the preview content for the currently selected file.
	/// The previous preview remains visible until the new one is ready.
	pub(crate) fn update_preview(&mut self) {
		if !self.preview_enabled {
			return;
		}

		let selection = match self.current_selection() {
			Some(SearchSelection::File(file)) => file,
			_ => {
				self.preview_content = PreviewContent::empty();
				self.preview_path.clear();
				self.pending_preview_path = None;
				self.preview_scroll = 0;
				return;
			}
		};

		let path = self.data.resolve_file_path(&selection);
		let path_str = path.display().to_string();

		// Skip if we already have this preview cached or it's already pending
		if self.preview_path == path_str {
			self.pending_preview_path = None;
			return;
		}
		if self.pending_preview_path.as_ref() == Some(&path_str) {
			return;
		}

		// Mark that we're loading this path, but keep displaying the old preview
		self.pending_preview_path = Some(path_str);

		// Request preview generation in background
		self.preview_runtime
			.request(path, self.bat_theme.clone(), 500);
	}

	/// Poll for completed preview results from the background worker.
	pub(crate) fn pump_preview_results(&mut self) {
		use std::sync::mpsc::TryRecvError;

		loop {
			match self.preview_runtime.try_recv() {
				Ok(result) => {
					// Only apply if this is still the current request
					if self.preview_runtime.is_current(result.id) {
						// Update the displayed preview and clear pending state
						self.preview_path = result.content.path.clone();
						self.preview_content = result.content;
						self.pending_preview_path = None;
						self.preview_scroll = 0;
						self.update_scrollbar_state();
					}
				}
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => break,
			}
		}
	}

	/// Scroll the preview pane up.
	pub(crate) fn scroll_preview_up(&mut self, lines: usize) {
		self.preview_scroll = self.preview_scroll.saturating_sub(lines);
		self.update_scrollbar_state();
	}

	/// Scroll the preview pane down.
	pub(crate) fn scroll_preview_down(&mut self, lines: usize) {
		let content_length = self.preview_content.line_count();
		let max_scroll = self.max_preview_scroll(content_length);
		self.preview_scroll = (self.preview_scroll + lines).min(max_scroll);
		self.update_scrollbar_state();
	}

	fn preview_viewport_len(&self, content_length: usize) -> usize {
		if content_length == 0 {
			0
		} else {
			self.preview_viewport_height.max(1).min(content_length)
		}
	}

	fn max_preview_scroll(&self, content_length: usize) -> usize {
		let viewport_len = self.preview_viewport_len(content_length);
		content_length.saturating_sub(viewport_len)
	}

	fn scrollbar_position(&self, scroll: usize, max_scroll: usize, content_length: usize) -> usize {
		if max_scroll == 0 || content_length == 0 {
			0
		} else {
			scroll.saturating_mul(content_length.saturating_sub(1)) / max_scroll
		}
	}

	pub(crate) fn update_preview_hover(&mut self, column: u16, row: u16) {
		if !self.preview_enabled {
			self.preview_hovered = false;
			return;
		}

		let Some(area) = self.preview_area else {
			self.preview_hovered = false;
			return;
		};

		let inside_x = column >= area.x && column < area.x.saturating_add(area.width);
		let inside_y = row >= area.y && row < area.y.saturating_add(area.height);
		self.preview_hovered = inside_x && inside_y;
	}

	pub(crate) fn update_results_hover(&mut self, column: u16, row: u16) {
		let Some(area) = self.results_area else {
			self.results_hovered = false;
			return;
		};

		let inside_x = column >= area.x && column < area.x.saturating_add(area.width);
		let inside_y = row >= area.y && row < area.y.saturating_add(area.height);
		self.results_hovered = inside_x && inside_y;
	}

	pub(crate) fn select_result_at(&mut self, _column: u16, row: u16) -> bool {
		let Some(area) = self.results_area else {
			return false;
		};

		// Table is rendered inside a rounded border block; subtract borders.
		let inner_y = area.y.saturating_add(1);
		let inner_width = area.width.saturating_sub(2);
		let inner_height = area.height.saturating_sub(2);
		if inner_width == 0 || inner_height == 0 {
			return false;
		}

		// Header row (1) + bottom margin (1) + separator (1) → rows start at y + 2.
		let body_start_y = inner_y.saturating_add(2);
		if row < body_start_y {
			return false;
		}

		let body_end_y = inner_y.saturating_add(inner_height);
		if row >= body_end_y {
			return false;
		}

		let row_in_view = row.saturating_sub(body_start_y) as usize;
		let visible_index = self.table_state.offset().saturating_add(row_in_view);

		if visible_index >= self.filtered_len() {
			return false;
		}

		self.table_state.select(Some(visible_index));
		true
	}

	/// Update scrollbar state to match current preview content and scroll position.
	pub(crate) fn update_scrollbar_state(&mut self) {
		let content_length = self.preview_content.line_count();
		let viewport_len = self.preview_viewport_len(content_length);
		let max_scroll = self.max_preview_scroll(content_length);
		self.preview_scroll = self.preview_scroll.min(max_scroll);
		let position = self.scrollbar_position(self.preview_scroll, max_scroll, content_length);

		self.preview_scrollbar_state = self
			.preview_scrollbar_state
			.content_length(content_length)
			.viewport_content_length(viewport_len)
			.position(position);
	}
}

#[cfg(test)]
mod tests {
	use std::time::{Duration, Instant};

	use frz_core::search_pipeline::{FileRow, MatchBatch, SearchViewV2};

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
