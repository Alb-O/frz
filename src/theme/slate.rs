use crate::theme::Theme;
use ratatui::style::{Color, Modifier, Style};

pub const SLATE: Theme = Theme {
    header: Style::new()
        .fg(Color::Rgb(226, 232, 240))
        .bg(Color::Rgb(15, 23, 42)),
    row_highlight: Style::new()
        .bg(Color::Rgb(30, 41, 59))
        .fg(Color::Rgb(250, 204, 21)),
    prompt: Style::new().fg(Color::LightCyan),
    empty: Style::new().fg(Color::DarkGray),
    highlight: Style::new()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD),
};
