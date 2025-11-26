use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use throbber_widgets_tui::{Throbber, ThrobberState};

use crate::ui::input::SearchInput;
use crate::ui::style::Theme;

/// Render metadata for a tab header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabItem<'a> {
	/// Text label displayed on the tab.
	pub label: &'a str,
}

/// Argument bundle for rendering the input area.
pub struct InputContext<'a> {
	/// The search input widget.
	pub search_input: &'a SearchInput<'a>,
	/// Title shown above the input.
	pub input_title: Option<&'a str>,
	/// Pane title if available.
	pub pane_title: Option<&'a str>,
	/// Tab headers to display.
	pub tabs: &'a [TabItem<'a>],
	/// Rendering area.
	pub area: Rect,
	/// Color theme.
	pub theme: &'a Theme,
}

/// Progress information for the prompt progress indicator.
pub struct ProgressState<'a> {
	/// Text describing the progress state.
	pub progress_text: &'a str,
	/// Whether the operation is complete.
	pub progress_complete: bool,
	/// Spinner animation state.
	pub throbber_state: &'a ThrobberState,
}

/// Render the input row with tabs at the right.
pub fn render_input_with_tabs(
	frame: &mut ratatui::Frame,
	input: InputContext<'_>,
	progress: ProgressState<'_>,
) {
	let InputContext {
		search_input,
		input_title,
		pane_title,
		tabs,
		area,
		theme,
	} = input;
	let ProgressState {
		progress_text,
		progress_complete,
		throbber_state,
	} = progress;

	let prompt = input_title.or(pane_title).unwrap_or("");
	let tabs_width = calculate_tabs_width(tabs);
	let prompt_width = calculate_prompt_width(prompt);

	let constraints = layout_constraints(!prompt.is_empty(), prompt_width, tabs_width);

	let horizontal = ratatui::layout::Layout::default()
		.direction(ratatui::layout::Direction::Horizontal)
		.constraints(constraints)
		.split(area);

	if !prompt.is_empty() {
		let prompt_text = format!("{} > ", prompt);
		let prompt_widget =
			ratatui::widgets::Paragraph::new(prompt_text).style(theme.prompt_style());
		frame.render_widget(prompt_widget, horizontal[0]);
	}

	let input_index = if prompt.is_empty() { 0 } else { 1 };
	let input_area = horizontal[input_index];
	search_input.render_textarea(frame, input_area);
	render_progress(
		frame,
		input_area,
		progress_text,
		progress_complete,
		throbber_state,
		theme,
	);

	let tabs_area = horizontal[horizontal.len() - 1];
	let tabs_inner = Rect {
		x: tabs_area.x.saturating_add(1),
		width: tabs_area.width.saturating_sub(1),
		..tabs_area
	};
	let selected = 0; // Only one tab now

	let tab_titles = build_tab_titles(theme, selected, tabs);

	let tabs = Tabs::new(tab_titles)
		.select(selected)
		.divider("")
		.padding("", " ")
		.highlight_style(theme.tab_highlight_style());

	frame.render_widget(tabs, tabs_inner);
}

fn calculate_prompt_width(prompt: &str) -> u16 {
	if prompt.is_empty() {
		0
	} else {
		prompt.len() as u16 + 3
	}
}

fn layout_constraints(
	has_prompt: bool,
	prompt_width: u16,
	tabs_width: u16,
) -> Vec<ratatui::layout::Constraint> {
	if has_prompt {
		vec![
			ratatui::layout::Constraint::Length(prompt_width),
			ratatui::layout::Constraint::Min(1),
			ratatui::layout::Constraint::Length(tabs_width),
		]
	} else {
		vec![
			ratatui::layout::Constraint::Min(1),
			ratatui::layout::Constraint::Length(tabs_width),
		]
	}
}

fn build_tab_titles(theme: &Theme, selected: usize, tabs: &[TabItem<'_>]) -> Vec<Line<'static>> {
	let active = theme.header_style();
	let inactive = theme.tab_inactive_style();
	tabs.iter()
		.enumerate()
		.map(|(index, tab)| {
			let label = format!(" {} ", tab.label);
			let style = if index == selected { active } else { inactive };
			Line::from(label).style(style)
		})
		.collect()
}

fn calculate_tabs_width(tabs: &[TabItem<'_>]) -> u16 {
	let mut width = 0u16;
	for tab in tabs {
		let label_len = tab.label.chars().count() as u16;
		width = width.saturating_add(label_len.saturating_add(3));
	}
	width.max(12)
}

fn render_progress(
	frame: &mut ratatui::Frame,
	area: Rect,
	progress_text: &str,
	progress_complete: bool,
	throbber_state: &ThrobberState,
	theme: &Theme,
) {
	if area.width == 0 || area.height == 0 || progress_text.is_empty() {
		return;
	}

	let muted_style = theme.empty_style();
	let label_span = Span::styled(progress_text.to_string(), muted_style);
	let mut line = Line::default();
	if !progress_complete {
		let spinner = Throbber::default()
			.style(muted_style)
			.throbber_style(muted_style);
		let spinner_span = spinner.to_symbol_span(throbber_state);
		line.spans.push(spinner_span);
	}
	line.spans.push(label_span);

	let line_width = line.width() as u16;
	if line_width == 0 {
		return;
	}

	let buffer = frame.buffer_mut();
	let mut start_x = if line_width >= area.width {
		area.left()
	} else {
		area.right().saturating_sub(line_width)
	};

	let input_row = area.top();
	let mut last_char_x: Option<u16> = None;
	for x in area.left()..area.right() {
		if let Some(cell) = buffer.cell((x, input_row))
			&& !cell.symbol().trim().is_empty()
		{
			last_char_x = Some(x);
		}
	}

	if let Some(last_x) = last_char_x {
		let min_start = last_x.saturating_add(3);
		if min_start > start_x {
			start_x = min_start;
		}
	}

	if start_x >= area.right() {
		return;
	}

	let max_width = area
		.right()
		.saturating_sub(start_x)
		.min(line_width)
		.min(area.width);

	if max_width == 0 {
		return;
	}

	buffer.set_line(start_x, input_row, &line, max_width);
}
