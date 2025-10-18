use crate::tui::theme::{Theme, ThemeDefinition};
use ratatui::style::{Color, Modifier, Style};

pub const NAME: &str = "solarized";

pub const SOLARIZED: Theme = Theme {
    header: Style::new()
        .fg(Color::Rgb(253, 246, 227))
        .bg(Color::Rgb(7, 54, 66)),
    row_highlight: Style::new()
        .bg(Color::Rgb(0, 43, 54))
        .fg(Color::Rgb(181, 137, 0)),
    prompt: Style::new().fg(Color::Rgb(38, 139, 210)),
    empty: Style::new().fg(Color::Rgb(88, 110, 117)),
    highlight: Style::new()
        .fg(Color::Rgb(181, 137, 0))
        .add_modifier(Modifier::BOLD),
};

pub const DEFINITION: ThemeDefinition = ThemeDefinition::new(NAME, SOLARIZED);
