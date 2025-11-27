//! ANSI parsing and bat highlighting utilities.

use std::path::Path;

use bat::assets::HighlightingAssets;
use bat::config::{Config, VisibleLines};
use bat::controller::Controller;
use bat::input::Input;
use bat::line_range::LineRanges;
use bat::style::{StyleComponent, StyleComponents};
use ratatui::style::Style;
use ratatui::text::{Line, Span};

/// Highlight file content using bat's Controller API.
pub fn highlight_with_bat(
	path: &Path,
	content: &str,
	bat_theme: Option<&str>,
	max_lines: usize,
	assets: &HighlightingAssets,
) -> Vec<Line<'static>> {
	let mut output = Vec::new();

	// Build bat Config
	let theme = bat_theme.unwrap_or("Monokai Extended").to_string();
	let mut style_components = StyleComponents::default();
	style_components.insert(StyleComponent::LineNumbers);

	let config = Config {
		colored_output: true,
		true_color: true,
		style_components,
		theme,
		visible_lines: VisibleLines::Ranges(LineRanges::all()),
		term_width: 120,
		tab_width: 4,
		..Default::default()
	};

	let controller = Controller::new(&config, assets);
	let input = Input::from_reader(Box::new(std::io::Cursor::new(content.to_string())))
		.with_name(Some(path));

	// Capture output to a string
	let mut buffer = String::new();
	if controller.run(vec![input], Some(&mut buffer)).is_ok() {
		for (i, line) in buffer.lines().enumerate() {
			if i >= max_lines {
				output.push(Line::from(Span::styled(
					"... (truncated)",
					Style::default(),
				)));
				break;
			}
			output.push(parse_ansi_line(line));
		}
	} else {
		// Fallback: render without highlighting
		for (i, line) in content.lines().enumerate() {
			if i >= max_lines {
				output.push(Line::from(Span::styled(
					"... (truncated)",
					Style::default(),
				)));
				break;
			}
			let line_num = format!("{:>4} │ ", i + 1);
			output.push(Line::from(vec![
				Span::styled(line_num, Style::default()),
				Span::raw(line.to_string()),
			]));
		}
	}

	output
}

/// Parse ANSI escape codes into ratatui spans.
///
/// This converts bat's ANSI output into ratatui's styled text format.
fn parse_ansi_line(line: &str) -> Line<'static> {
	let mut spans = Vec::new();
	let mut current_text = String::new();
	let mut current_style = Style::default();
	let mut chars = line.chars().peekable();

	while let Some(ch) = chars.next() {
		if ch == '\x1b' {
			// Start of ANSI escape sequence
			if !current_text.is_empty() {
				spans.push(Span::styled(
					std::mem::take(&mut current_text),
					current_style,
				));
			}

			// Parse escape sequence
			if chars.next() == Some('[') {
				let mut code = String::new();
				while let Some(&c) = chars.peek() {
					if c.is_ascii_digit() || c == ';' {
						code.push(chars.next().unwrap());
					} else {
						break;
					}
				}

				// Consume the final character (usually 'm')
				if chars.next() == Some('m') {
					current_style = parse_ansi_codes(&code, current_style);
				}
			}
		} else {
			current_text.push(ch);
		}
	}

	if !current_text.is_empty() {
		spans.push(Span::styled(current_text, current_style));
	}

	Line::from(spans)
}

/// Parse ANSI SGR codes and update style.
fn parse_ansi_codes(codes: &str, mut style: Style) -> Style {
	use ratatui::style::{Color, Modifier};

	let parts: Vec<&str> = codes.split(';').collect();
	let mut i = 0;

	while i < parts.len() {
		match parts[i].parse::<u8>() {
			Ok(0) => style = Style::default(),
			Ok(1) => style = style.add_modifier(Modifier::BOLD),
			Ok(2) => style = style.add_modifier(Modifier::DIM),
			Ok(3) => style = style.add_modifier(Modifier::ITALIC),
			Ok(4) => style = style.add_modifier(Modifier::UNDERLINED),
			Ok(7) => style = style.add_modifier(Modifier::REVERSED),
			Ok(22) => {
				style = style
					.remove_modifier(Modifier::BOLD)
					.remove_modifier(Modifier::DIM)
			}
			Ok(23) => style = style.remove_modifier(Modifier::ITALIC),
			Ok(24) => style = style.remove_modifier(Modifier::UNDERLINED),
			Ok(27) => style = style.remove_modifier(Modifier::REVERSED),
			// Foreground colors (30-37)
			Ok(30) => style = style.fg(Color::Black),
			Ok(31) => style = style.fg(Color::Red),
			Ok(32) => style = style.fg(Color::Green),
			Ok(33) => style = style.fg(Color::Yellow),
			Ok(34) => style = style.fg(Color::Blue),
			Ok(35) => style = style.fg(Color::Magenta),
			Ok(36) => style = style.fg(Color::Cyan),
			Ok(37) => style = style.fg(Color::Gray),
			Ok(39) => style = style.fg(Color::Reset),
			// Bright foreground colors (90-97)
			Ok(90) => style = style.fg(Color::DarkGray),
			Ok(91) => style = style.fg(Color::LightRed),
			Ok(92) => style = style.fg(Color::LightGreen),
			Ok(93) => style = style.fg(Color::LightYellow),
			Ok(94) => style = style.fg(Color::LightBlue),
			Ok(95) => style = style.fg(Color::LightMagenta),
			Ok(96) => style = style.fg(Color::LightCyan),
			Ok(97) => style = style.fg(Color::White),
			// Background colors (40-47)
			Ok(40) => style = style.bg(Color::Black),
			Ok(41) => style = style.bg(Color::Red),
			Ok(42) => style = style.bg(Color::Green),
			Ok(43) => style = style.bg(Color::Yellow),
			Ok(44) => style = style.bg(Color::Blue),
			Ok(45) => style = style.bg(Color::Magenta),
			Ok(46) => style = style.bg(Color::Cyan),
			Ok(47) => style = style.bg(Color::Gray),
			Ok(49) => style = style.bg(Color::Reset),
			// Bright background colors (100-107)
			Ok(100) => style = style.bg(Color::DarkGray),
			Ok(101) => style = style.bg(Color::LightRed),
			Ok(102) => style = style.bg(Color::LightGreen),
			Ok(103) => style = style.bg(Color::LightYellow),
			Ok(104) => style = style.bg(Color::LightBlue),
			Ok(105) => style = style.bg(Color::LightMagenta),
			Ok(106) => style = style.bg(Color::LightCyan),
			Ok(107) => style = style.bg(Color::White),
			// 256-color mode (38;5;N or 48;5;N)
			Ok(38) if i + 2 < parts.len() && parts[i + 1] == "5" => {
				if let Ok(n) = parts[i + 2].parse::<u8>() {
					style = style.fg(Color::Indexed(n));
					i += 2;
				}
			}
			Ok(48) if i + 2 < parts.len() && parts[i + 1] == "5" => {
				if let Ok(n) = parts[i + 2].parse::<u8>() {
					style = style.bg(Color::Indexed(n));
					i += 2;
				}
			}
			// True color mode (38;2;R;G;B or 48;2;R;G;B)
			Ok(38) if i + 4 < parts.len() && parts[i + 1] == "2" => {
				if let (Ok(r), Ok(g), Ok(b)) = (
					parts[i + 2].parse::<u8>(),
					parts[i + 3].parse::<u8>(),
					parts[i + 4].parse::<u8>(),
				) {
					style = style.fg(Color::Rgb(r, g, b));
					i += 4;
				}
			}
			Ok(48) if i + 4 < parts.len() && parts[i + 1] == "2" => {
				if let (Ok(r), Ok(g), Ok(b)) = (
					parts[i + 2].parse::<u8>(),
					parts[i + 3].parse::<u8>(),
					parts[i + 4].parse::<u8>(),
				) {
					style = style.bg(Color::Rgb(r, g, b));
					i += 4;
				}
			}
			_ => {}
		}
		i += 1;
	}

	style
}
