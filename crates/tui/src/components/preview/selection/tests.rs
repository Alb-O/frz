use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;

use super::extract::extract_selected_text;
use super::highlight::{apply_selection_to_lines, selection_style};
use super::state::TextSelection;
use crate::style::Theme;

fn test_theme() -> Theme {
	Theme {
		header: Style::default(),
		row_highlight: Style::default().bg(Color::LightBlue).fg(Color::Black),
		prompt: Style::default(),
		empty: Style::default(),
		highlight: Style::default(),
	}
}

#[test]
fn test_selection_start_update_finish() {
	let mut sel = TextSelection::new();
	assert!(!sel.has_selection());

	sel.start(10, 5, 0);
	assert!(sel.selecting);
	assert!(!sel.active);

	sel.update(20, 5, 0);
	assert_eq!(sel.focus, Some((20, 5)));

	sel.finish();
	assert!(!sel.selecting);
	assert!(sel.active);
}

#[test]
fn test_click_without_drag_not_selection() {
	let mut sel = TextSelection::new();
	sel.start(10, 5, 0);
	sel.finish();
	assert!(!sel.active);
}

#[test]
fn test_normalized_bounds() {
	let area = Rect::new(5, 5, 50, 20);

	let mut sel = TextSelection::new();
	sel.anchor = Some((10, 7));
	sel.focus = Some((20, 7));
	sel.anchor_scroll = 0;
	sel.focus_scroll = 0;
	sel.active = true;

	let bounds = sel.normalized_bounds(area).unwrap();
	assert_eq!(bounds, ((5, 2), (15, 2)));

	sel.anchor = Some((20, 7));
	sel.focus = Some((10, 7));
	let bounds = sel.normalized_bounds(area).unwrap();
	assert_eq!(bounds, ((5, 2), (15, 2)));
}

#[test]
fn normalized_bounds_respects_scroll() {
	let area = Rect::new(0, 0, 80, 10);
	let mut sel = TextSelection::new();
	sel.anchor = Some((5, 3));
	sel.focus = Some((10, 4));
	sel.anchor_scroll = 10;
	sel.focus_scroll = 10;
	let bounds = sel.normalized_bounds(area).unwrap();
	assert_eq!(bounds, ((5, 13), (10, 14)));
}

#[test]
fn gutter_is_not_selected_on_full_line() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((10, 2)),
		anchor_scroll: 0,
		focus_scroll: 0,
		selecting: false,
		active: true,
	};

	let area = Rect::new(0, 0, 20, 5);
	let lines = vec![
		Line::from(" 1 │ first"),
		Line::from(" 2 │ second"),
		Line::from(" 3 │ third"),
	];

	let theme = test_theme();
	let highlighted = apply_selection_to_lines(&lines, &selection, area, &theme);
	let sel_style = selection_style(&theme);
	for line in highlighted {
		let first_span = line.spans.first().expect("gutter span");
		assert_ne!(first_span.style, sel_style, "gutter should not be selected");
		assert!(first_span.content.starts_with(' '));
	}
}

#[test]
fn gutter_without_separator_not_selected() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((10, 1)),
		anchor_scroll: 0,
		focus_scroll: 0,
		selecting: false,
		active: true,
	};
	let area = Rect::new(0, 0, 20, 5);
	let lines = vec![Line::from(" 12  hello world")];

	let theme = test_theme();
	let highlighted = apply_selection_to_lines(&lines, &selection, area, &theme);
	let sel_style = selection_style(&theme);
	let first_span = highlighted[0].spans.first().expect("gutter span");
	assert_ne!(first_span.style, sel_style, "gutter should not be selected");
	assert!(first_span.content.starts_with(' '));
}

#[test]
fn selection_respects_scroll_offset() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((4, 0)),
		anchor_scroll: 5,
		focus_scroll: 5,
		selecting: false,
		active: true,
	};
	let area = Rect::new(0, 0, 20, 5);
	let lines = (0..8)
		.map(|i| Line::from(format!("{i:02} content")))
		.collect::<Vec<_>>();

	let theme = test_theme();
	let highlighted = apply_selection_to_lines(&lines, &selection, area, &theme);
	let sel_style = selection_style(&theme);

	for (idx, line) in highlighted.iter().enumerate() {
		let has_selected = line.spans.iter().any(|s| s.style == sel_style);
		if idx == 5 {
			assert!(has_selected, "line at scroll offset should be selected");
		} else {
			assert!(!has_selected, "only scrolled line should be selected");
		}
	}
}

#[test]
fn selection_preserves_indentation() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((12, 0)),
		anchor_scroll: 0,
		focus_scroll: 0,
		selecting: false,
		active: true,
	};

	let area = Rect::new(0, 0, 40, 5);
	let lines = vec![Line::from("    let x = 1;")];

	let text = extract_selected_text(&lines, &selection, area).expect("text");
	assert!(text.starts_with("    "), "indentation should be preserved");
}

#[test]
fn highlighting_includes_indentation_without_separator() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((12, 0)),
		anchor_scroll: 0,
		focus_scroll: 0,
		selecting: false,
		active: true,
	};

	let area = Rect::new(0, 0, 40, 5);
	let lines = vec![Line::from("12  \t  let x = 1;")];
	let theme = test_theme();
	let highlighted = apply_selection_to_lines(&lines, &selection, area, &theme);
	let sel_style = selection_style(&theme);

	let selected_span = highlighted[0].spans.iter().find(|s| s.style == sel_style);
	let Some(selected_span) = selected_span else {
		panic!("expected selection highlight");
	};
	let starts_with_indent = selected_span
		.content
		.chars()
		.next()
		.map(|c| c == ' ' || c == '\t')
		.unwrap_or(false);
	assert!(
		starts_with_indent,
		"indentation (after gutter) should be part of selection highlight"
	);
}

#[test]
fn wrapped_continuation_does_not_insert_extra_newline() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((10, 1)),
		anchor_scroll: 0,
		focus_scroll: 0,
		selecting: false,
		active: true,
	};

	let area = Rect::new(0, 0, 40, 5);
	let lines = vec![Line::from(" 1 │ hello "), Line::from("    world")];

	let text = extract_selected_text(&lines, &selection, area).expect("text");
	assert_eq!(text, "hello world", "wrapped lines should be unwrapped");
}

#[test]
fn continuation_highlight_skips_gutter() {
	let selection = TextSelection {
		anchor: Some((0, 0)),
		focus: Some((5, 1)),
		anchor_scroll: 0,
		focus_scroll: 0,
		selecting: false,
		active: true,
	};

	let area = Rect::new(0, 0, 40, 5);
	let lines = vec![Line::from(" 1 │ hello "), Line::from("    world")];
	let theme = test_theme();
	let highlighted = apply_selection_to_lines(&lines, &selection, area, &theme);
	let sel_style = selection_style(&theme);

	let first_span = highlighted[1].spans.first().expect("continuation span");
	assert_ne!(first_span.style, sel_style);
}
