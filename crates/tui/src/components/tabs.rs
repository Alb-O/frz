use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use throbber_widgets_tui::{Throbber, ThrobberState};

use crate::input::SearchInput;
use crate::style::Theme;

/// Argument bundle for rendering the input area.
pub struct InputContext<'a> {
	/// The search input widget.
	pub search_input: &'a SearchInput<'a>,
	/// Placeholder text shown when input is empty.
	pub placeholder: Option<&'a str>,
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

/// Render the input row with optional placeholder.
pub fn render_input(
	frame: &mut ratatui::Frame,
	input: InputContext<'_>,
	progress: ProgressState<'_>,
) {
	let InputContext {
		search_input,
		placeholder,
		area,
		theme,
	} = input;
	let ProgressState {
		progress_text,
		progress_complete,
		throbber_state,
	} = progress;

	search_input.render_textarea(frame, area);

	// Placeholder text if input is empty
	let input_text = search_input.text();
	if input_text.is_empty()
		&& let Some(placeholder_text) = placeholder
	{
		render_placeholder(frame, area, placeholder_text, theme);
	}

	render_progress(
		frame,
		area,
		progress_text,
		progress_complete,
		throbber_state,
		theme,
	);
}

fn render_placeholder(frame: &mut ratatui::Frame, area: Rect, text: &str, theme: &Theme) {
	if area.width == 0 || area.height == 0 || text.is_empty() {
		return;
	}
	let dimmed_style = theme.empty_style();
	let available_width = area.width as usize;
	let display_text: String = text.chars().take(available_width).collect();
	let buffer = frame.buffer_mut();
	buffer.set_line(
		area.left(),
		area.top(),
		&Line::from(Span::styled(display_text, dimmed_style)),
		area.width,
	);
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
