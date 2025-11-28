use ratatui::style::{Color, Style};

/// A theme containing styles for various UI elements.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
	/// Style for header elements.
	pub header: Style,
	/// Style for highlighted rows.
	pub row_highlight: Style,
	/// Style for prompt elements.
	pub prompt: Style,
	/// Style for empty states.
	pub empty: Style,
	/// Style for highlighted elements.
	pub highlight: Style,
}

impl Theme {
	/// Returns the style for inactive tabs.
	#[must_use]
	pub fn tab_inactive_style(&self) -> Style {
		Style::new()
			.fg(self.header.fg.unwrap_or(Color::Reset))
			.bg(self.row_highlight.bg.unwrap_or(Color::Reset))
	}

	/// Returns the style for highlighted tabs.
	#[must_use]
	pub fn tab_highlight_style(&self) -> Style {
		Style::new().bg(self.header.bg.unwrap_or(Color::Reset))
	}
}

/// Describes a theme instance that can be registered with the UI.
#[derive(Debug, Clone)]
pub struct ThemeRegistration {
	/// The name of the theme.
	pub name: String,
	/// The theme configuration.
	pub theme: Theme,
	/// Alternate names for the theme.
	pub aliases: Vec<String>,
	/// Optional bat syntax highlighting theme name.
	pub bat_theme: Option<String>,
}

impl ThemeRegistration {
	/// Creates a new theme registration with the given name and theme.
	pub fn new(name: impl Into<String>, theme: Theme) -> Self {
		Self {
			name: name.into(),
			theme,
			aliases: Vec::new(),
			bat_theme: None,
		}
	}

	/// Adds a single alias to this theme registration.
	pub fn alias(mut self, alias: impl Into<String>) -> Self {
		self.aliases.push(alias.into());
		self
	}

	/// Sets the bat syntax highlighting theme name.
	pub fn with_bat_theme(mut self, bat_theme: impl Into<String>) -> Self {
		self.bat_theme = Some(bat_theme.into());
		self
	}

	/// Adds multiple aliases to this theme registration.
	pub fn aliases<I, S>(mut self, aliases: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.aliases.extend(aliases.into_iter().map(Into::into));
		self
	}
}

/// Summary of the operations performed while registering themes.
#[derive(Debug, Default, Clone)]
pub struct ThemeRegistrationReport {
	/// Names of themes that were newly inserted.
	pub inserted: Vec<String>,
	/// Names of themes that were replaced.
	pub replaced: Vec<String>,
	/// Aliases that could not be registered due to conflicts.
	pub alias_conflicts: Vec<AliasConflict>,
}

impl ThemeRegistrationReport {
	/// Returns `true` if no operations were performed during registration.
	#[must_use]
	pub fn is_clean(&self) -> bool {
		self.inserted.is_empty() && self.replaced.is_empty() && self.alias_conflicts.is_empty()
	}
}

/// Describes an alias that could not be registered because it targets multiple themes.
#[derive(Debug, Clone)]
pub struct AliasConflict {
	/// The conflicting alias name.
	pub alias: String,
	/// The name of the existing theme using this alias.
	pub existing: String,
	/// The name of the theme that attempted to use this alias.
	pub attempted: String,
}

/// Snapshot of a registered theme and its metadata.
#[derive(Debug, Clone)]
pub struct ThemeDescriptor {
	/// The name of the theme.
	pub name: String,
	/// Alternate names for the theme.
	pub aliases: Vec<String>,
	/// The theme configuration.
	pub theme: Theme,
	/// Optional bat syntax highlighting theme name.
	pub bat_theme: Option<String>,
}
