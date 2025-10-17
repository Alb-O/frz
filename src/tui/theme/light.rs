use super::Theme;
use ratatui::style::{Color, Modifier, Style};

pub const LIGHT: Theme = Theme {
    header: Style::new()
        .fg(Color::Rgb(15, 23, 42))
        .bg(Color::Rgb(226, 232, 240)),
    row_highlight: Style::new()
        .bg(Color::Rgb(200, 200, 200))
        .fg(Color::Rgb(120, 120, 0)),
    prompt: Style::new().fg(Color::Rgb(0, 102, 153)),
    empty: Style::new().fg(Color::Rgb(100, 100, 100)),
    highlight: Style::new()
        .fg(Color::Rgb(120, 120, 0))
        .add_modifier(Modifier::BOLD),
};
