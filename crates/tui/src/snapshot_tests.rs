use std::path::Path;

use frz_core::search_pipeline::SearchData;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;

use crate::App;
use crate::components::PreviewContent;
use crate::components::preview::highlight::highlight_with_bat;

#[test]
fn preview_wrap_respects_gutter_and_indent_snapshot() {
	let data = SearchData::new();
	let mut app = App::new(data);
	app.preview_enabled = true;
	let assets = bat::assets::HighlightingAssets::from_binary();
	let content = r#"
fn long_function_name(arg_one: usize, arg_two: usize, arg_three: usize) {
    do_something_complicated(arg_one, arg_two, arg_three);
    do_something_even_more_complicated();
}

println!("still indented");
"#
	.trim_start_matches('\n');
	let highlighted = highlight_with_bat(Path::new("wrap_example.rs"), content, None, 256, &assets);
	app.preview_content = PreviewContent::text("wrap_example.rs", highlighted);

	let backend = TestBackend::new(120, 32);
	let mut terminal = Terminal::new(backend).expect("terminal");
	terminal
		.draw(|frame| app.draw(frame))
		.expect("draw snapshot frame");

	let snapshot = buffer_to_string(terminal.backend().buffer());
	let gutter_width = detect_gutter_width(&snapshot);
	assert_no_gutter_overlap(&snapshot, gutter_width);
	assert_continuations_indented(&snapshot, gutter_width);
	insta::assert_snapshot!("preview_wrap_respects_gutter_and_indent", snapshot);
}

fn buffer_to_string(buf: &Buffer) -> String {
	let mut lines = Vec::new();
	for y in 0..buf.area.height {
		let mut line = String::new();
		for x in 0..buf.area.width {
			line.push_str(buf[(x, y)].symbol());
		}
		lines.push(line);
	}
	lines.join("\n")
}

fn detect_gutter_width(snapshot: &str) -> usize {
	for line in snapshot.lines() {
		let Some(preview) = extract_preview(line) else {
			continue;
		};
		if !preview.trim_start().starts_with(char::is_numeric) {
			continue;
		}

		let width = preview
			.chars()
			.take_while(|ch| ch.is_ascii_digit() || *ch == ' ')
			.count();
		if width > 0 {
			return width;
		}
	}
	7
}

fn assert_no_gutter_overlap(snapshot: &str, gutter_width: usize) {
	for (idx, line) in snapshot.lines().enumerate() {
		let Some(preview) = extract_preview(line) else {
			continue;
		};

		let is_preview_line = preview.contains('│')
			&& (preview.contains("fn")
				|| preview.contains("println!")
				|| preview.contains("do_something"));
		if !is_preview_line {
			continue;
		}

		for ch in preview.chars().take(gutter_width) {
			assert!(
				ch == ' ' || ch == '│' || ch.is_ascii_digit(),
				"gutter overlap on line {idx}: found '{ch}' in gutter\nline: {preview:?}"
			);
		}
	}
}

fn assert_continuations_indented(snapshot: &str, gutter_width: usize) {
	let mut last_indent = 0;

	for (idx, line) in snapshot.lines().enumerate() {
		let Some(preview) = extract_preview(line) else {
			continue;
		};

		let trimmed_start = preview.trim_start();
		let starts_with_digit = trimmed_start.chars().next().map(|c| c.is_ascii_digit());

		match starts_with_digit {
			Some(true) => {
				let after_gutter = preview.chars().skip(gutter_width).collect::<String>();
				let indent = after_gutter.chars().take_while(|c| *c == ' ').count();
				last_indent = indent;
			}
			Some(false) => {
				if preview.trim().is_empty() {
					continue;
				}

				let first_non_space = preview
					.char_indices()
					.find(|(_, ch)| *ch != ' ')
					.map(|(i, _)| i)
					.unwrap_or(0);

				assert!(
					first_non_space >= gutter_width + last_indent,
					"continuation not indented on line {idx}: starts at column {first_non_space}\nline: {preview:?}"
				);
			}
			None => {}
		}
	}
}

fn extract_preview(line: &str) -> Option<&str> {
	let (_, right) = line.split_once("││")?;
	let trimmed = right.trim_end();
	let without_border = trimmed
		.strip_suffix('│')
		.map(str::trim_end)
		.unwrap_or(trimmed);
	Some(without_border)
}
