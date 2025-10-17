use ratatui::style::{Color, Style};

macro_rules! declare_themes {
    ( $( ($mod:ident, $const:ident) ),* $(,)? ) => {
        $( pub mod $mod; )*
        $( pub use $mod::$const; )*

        /// Canonical theme names supported by the UI.
        pub const NAMES: &[&str] = &[ $( stringify!($mod) ),* ];

        /// Lookup a Theme by case-insensitive name.
        pub fn by_name(name: &str) -> Option<Theme> {
            match name.to_lowercase().as_str() {
                $( stringify!($mod) => Some($const), )*
                _ => None,
            }
        }
    };
}

declare_themes!((slate, SLATE), (solarized, SOLARIZED), (light, LIGHT),);

/// Core Theme struct
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub header: Style,
    pub row_highlight: Style,
    pub prompt: Style,
    pub empty: Style,
    pub highlight: Style,
}

impl Default for Theme {
    fn default() -> Self {
        SLATE
    }
}

impl Theme {
    #[must_use]
    pub fn header_style(&self) -> Style {
        self.header
    }

    #[must_use]
    pub fn row_highlight_style(&self) -> Style {
        self.row_highlight
    }

    #[must_use]
    pub fn prompt_style(&self) -> Style {
        self.prompt
    }

    #[must_use]
    pub fn empty_style(&self) -> Style {
        self.empty
    }

    #[must_use]
    pub fn highlight_style(&self) -> Style {
        self.highlight
    }

    #[must_use]
    pub fn header_fg(&self) -> Color {
        self.header.fg.unwrap_or(Color::Reset)
    }

    #[must_use]
    pub fn header_bg(&self) -> Color {
        self.header.bg.unwrap_or(Color::Reset)
    }

    #[must_use]
    pub fn row_highlight_bg(&self) -> Color {
        self.row_highlight.bg.unwrap_or(Color::Reset)
    }

    #[must_use]
    pub fn tab_inactive_style(&self) -> Style {
        Style::new()
            .fg(self.header_fg())
            .bg(self.row_highlight_bg())
    }

    #[must_use]
    pub fn tab_highlight_style(&self) -> Style {
        Style::new().bg(self.header_bg())
    }
}
