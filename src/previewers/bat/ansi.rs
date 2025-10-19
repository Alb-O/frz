use std::mem;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

/// Converts ANSI-coloured output from `bat` into `ratatui` [`Text`] data.
///
/// `bat` emits raw strings with escape sequences. The previewer needs to
/// translate them into styled spans to render properly inside the TUI.
pub(super) fn ansi_to_text(input: &str) -> Text<'static> {
    let mut lines = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut buffer = String::new();
    let mut state = AnsiStyleState::default();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if handle_escape_sequence(&mut chars, &mut buffer, &mut state, &mut current_line) {
                continue;
            }
            // If the escape sequence was not recognised, drop the escape byte.
            continue;
        }

        match ch {
            '\r' => {}
            '\n' => {
                flush_buffer(&mut buffer, &state, &mut current_line);
                if current_line.is_empty() {
                    lines.push(Line::default());
                } else {
                    lines.push(Line::from(mem::take(&mut current_line)));
                }
            }
            ch if ch.is_control() => {}
            _ => buffer.push(ch),
        }
    }

    flush_buffer(&mut buffer, &state, &mut current_line);
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    } else if lines.is_empty() {
        lines.push(Line::default());
    }

    Text::from(lines)
}

fn flush_buffer(buffer: &mut String, state: &AnsiStyleState, line: &mut Vec<Span<'static>>) {
    if buffer.is_empty() {
        return;
    }

    let content = mem::take(buffer);
    line.push(Span::styled(content, state.to_style()));
}

#[derive(Default, Clone)]
struct AnsiStyleState {
    foreground: Option<Color>,
    background: Option<Color>,
    modifiers: Modifier,
}

impl AnsiStyleState {
    fn reset(&mut self) {
        self.foreground = None;
        self.background = None;
        self.modifiers = Modifier::empty();
    }

    fn to_style(&self) -> Style {
        let mut style = Style::default();
        if let Some(color) = self.foreground {
            style = style.fg(color);
        }
        if let Some(color) = self.background {
            style = style.bg(color);
        }
        if !self.modifiers.is_empty() {
            style = style.add_modifier(self.modifiers);
        }
        style
    }
}

fn apply_sgr_sequence(state: &mut AnsiStyleState, params: &str) {
    let mut values: Vec<i64> = if params.is_empty() {
        vec![0]
    } else {
        params
            .split(';')
            .map(|part| part.parse::<i64>().unwrap_or(0))
            .collect()
    };

    if values.is_empty() {
        values.push(0);
    }

    let mut index = 0;
    while index < values.len() {
        match values[index] {
            0 => state.reset(),
            1 => state.modifiers.insert(Modifier::BOLD),
            2 => state.modifiers.insert(Modifier::DIM),
            3 => state.modifiers.insert(Modifier::ITALIC),
            4 => state.modifiers.insert(Modifier::UNDERLINED),
            7 => state.modifiers.insert(Modifier::REVERSED),
            21 | 22 => state.modifiers.remove(Modifier::BOLD | Modifier::DIM),
            23 => state.modifiers.remove(Modifier::ITALIC),
            24 => state.modifiers.remove(Modifier::UNDERLINED),
            27 => state.modifiers.remove(Modifier::REVERSED),
            30..=37 => {
                if let Some(color) = map_standard_color(values[index] - 30, false) {
                    state.foreground = Some(color);
                }
            }
            90..=97 => {
                if let Some(color) = map_standard_color(values[index] - 90, true) {
                    state.foreground = Some(color);
                }
            }
            40..=47 => {
                if let Some(color) = map_standard_color(values[index] - 40, false) {
                    state.background = Some(color);
                }
            }
            100..=107 => {
                if let Some(color) = map_standard_color(values[index] - 100, true) {
                    state.background = Some(color);
                }
            }
            38 => {
                let consumed = apply_extended_color(&values[index + 1..], &mut state.foreground);
                index += consumed;
            }
            48 => {
                let consumed = apply_extended_color(&values[index + 1..], &mut state.background);
                index += consumed;
            }
            39 => state.foreground = None,
            49 => state.background = None,
            _ => {}
        }

        index += 1;
    }
}

fn map_standard_color(index: i64, bright: bool) -> Option<Color> {
    let color = match (bright, index) {
        (false, 0) => Color::Black,
        (false, 1) => Color::Red,
        (false, 2) => Color::Green,
        (false, 3) => Color::Yellow,
        (false, 4) => Color::Blue,
        (false, 5) => Color::Magenta,
        (false, 6) => Color::Cyan,
        (false, 7) => Color::Gray,
        (true, 0) => Color::DarkGray,
        (true, 1) => Color::LightRed,
        (true, 2) => Color::LightGreen,
        (true, 3) => Color::LightYellow,
        (true, 4) => Color::LightBlue,
        (true, 5) => Color::LightMagenta,
        (true, 6) => Color::LightCyan,
        (true, 7) => Color::White,
        _ => return None,
    };
    Some(color)
}

fn apply_extended_color(params: &[i64], target: &mut Option<Color>) -> usize {
    if params.is_empty() {
        return 0;
    }

    match params[0] {
        2 if params.len() >= 4 => {
            let r = clamp_to_u8(params[1]);
            let g = clamp_to_u8(params[2]);
            let b = clamp_to_u8(params[3]);
            *target = Some(Color::Rgb(r, g, b));
            4
        }
        5 if params.len() >= 2 => {
            let index = clamp_to_u8(params[1]);
            *target = Some(Color::Indexed(index));
            2
        }
        _ => 0,
    }
}

fn clamp_to_u8(value: i64) -> u8 {
    value.clamp(0, 255) as u8
}

fn handle_escape_sequence(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    buffer: &mut String,
    state: &mut AnsiStyleState,
    current_line: &mut Vec<Span<'static>>,
) -> bool {
    let Some(indicator) = chars.peek().copied() else {
        return true;
    };

    match indicator {
        '[' => {
            chars.next();
            let mut sequence = String::new();
            for next in chars.by_ref() {
                sequence.push(next);
                if ('@'..='~').contains(&next) {
                    break;
                }
            }

            if let Some(code) = sequence.chars().last()
                && code == 'm'
            {
                flush_buffer(buffer, state, current_line);
                let params = &sequence[..sequence.len().saturating_sub(1)];
                apply_sgr_sequence(state, params);
            }

            true
        }
        ']' => {
            chars.next();
            consume_osc_sequence(chars);
            true
        }
        'P' | '^' | '_' | 'X' => {
            chars.next();
            consume_st_terminated_sequence(chars);
            true
        }
        '(' | ')' | '*' | '+' | '-' | '.' | '/' => {
            chars.next();
            // Character set selection sequences have a single final byte.
            chars.next();
            true
        }
        _ => {
            // Consume the parameter/final byte for single-character escapes such as ESCc.
            chars.next();
            true
        }
    }
}

fn consume_osc_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(next) = chars.next() {
        match next {
            '\u{7}' => break,
            '\u{1b}' => {
                if matches!(chars.peek(), Some('\\')) {
                    chars.next();
                    break;
                }
            }
            _ => {}
        }
    }
}

fn consume_st_terminated_sequence(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(next) = chars.next() {
        if next == '\u{1b}' && matches!(chars.peek(), Some('\\')) {
            chars.next();
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn parses_basic_sgr_sequences() {
        let text = ansi_to_text("\u{1b}[31mred\u{1b}[0m");
        assert_eq!(flatten_text(&text), "red");
        assert_eq!(text.lines.len(), 1);
        let span = &text.lines[0].spans[0];
        assert_eq!(span.style.fg, Some(Color::Red));
    }

    #[test]
    fn strips_osc_sequences() {
        let text =
            ansi_to_text("pre\u{1b}]8;;https://example.com\u{1b}\\mid\u{1b}]8;;\u{1b}\\post");
        assert_eq!(flatten_text(&text), "premidpost");
    }

    #[test]
    fn strips_st_terminated_sequences() {
        let text = ansi_to_text("a\u{1b}Pignored\u{1b}\\b");
        assert_eq!(flatten_text(&text), "ab");
    }

    #[test]
    fn drops_other_control_characters() {
        let text = ansi_to_text("foo\u{7}bar\u{c}");
        assert_eq!(flatten_text(&text), "foobar");
    }

    #[test]
    fn removes_single_character_escape_sequences() {
        let text = ansi_to_text("start\u{1b}cend");
        assert_eq!(flatten_text(&text), "startend");
    }

    fn flatten_text(text: &Text<'_>) -> String {
        let mut result = String::new();
        for (index, line) in text.lines.iter().enumerate() {
            for span in &line.spans {
                result.push_str(span.content.as_ref());
            }
            if index + 1 != text.lines.len() {
                result.push('\n');
            }
        }
        result
    }
}
