use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
	pub header: Style,
	pub row_highlight: Style,
	pub prompt: Style,
	pub empty: Style,
	pub highlight: Style,
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

/// Definition for a built-in theme bundled with the application.
#[derive(Debug, Clone, Copy)]
pub struct ThemeDefinition {
	pub name: &'static str,
	pub theme: Theme,
	pub aliases: &'static [&'static str],
}

impl ThemeDefinition {
	pub const fn new(name: &'static str, theme: Theme) -> Self {
		Self {
			name,
			theme,
			aliases: &[],
		}
	}

	pub const fn with_aliases(mut self, aliases: &'static [&'static str]) -> Self {
		self.aliases = aliases;
		self
	}

	pub fn to_registration(self) -> ThemeRegistration {
		ThemeRegistration {
			name: self.name.to_owned(),
			theme: self.theme,
			aliases: self.aliases.iter().map(|alias| alias.to_string()).collect(),
			bat_theme: None,
		}
	}
}

/// Describes a theme instance that can be registered with the UI.
#[derive(Debug, Clone)]
pub struct ThemeRegistration {
	pub name: String,
	pub theme: Theme,
	pub aliases: Vec<String>,
	pub bat_theme: Option<String>,
}

impl ThemeRegistration {
	pub fn new(name: impl Into<String>, theme: Theme) -> Self {
		Self {
			name: name.into(),
			theme,
			aliases: Vec::new(),
			bat_theme: None,
		}
	}

	pub fn alias(mut self, alias: impl Into<String>) -> Self {
		self.aliases.push(alias.into());
		self
	}

	pub fn with_bat_theme(mut self, bat_theme: impl Into<String>) -> Self {
		self.bat_theme = Some(bat_theme.into());
		self
	}

	pub fn aliases<I, S>(mut self, aliases: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: Into<String>,
	{
		self.aliases.extend(aliases.into_iter().map(Into::into));
		self
	}
}

impl From<ThemeDefinition> for ThemeRegistration {
	fn from(definition: ThemeDefinition) -> Self {
		definition.to_registration()
	}
}

/// Summary of the operations performed while registering themes.
#[derive(Debug, Default, Clone)]
pub struct ThemeRegistrationReport {
	pub inserted: Vec<String>,
	pub replaced: Vec<String>,
	pub alias_conflicts: Vec<AliasConflict>,
}

impl ThemeRegistrationReport {
	#[must_use]
	pub fn is_clean(&self) -> bool {
		self.inserted.is_empty() && self.replaced.is_empty() && self.alias_conflicts.is_empty()
	}
}

/// Describes an alias that could not be registered because it targets multiple themes.
#[derive(Debug, Clone)]
pub struct AliasConflict {
	pub alias: String,
	pub existing: String,
	pub attempted: String,
}

/// Snapshot of a registered theme and its metadata.
#[derive(Debug, Clone)]
pub struct ThemeDescriptor {
	pub name: String,
	pub aliases: Vec<String>,
	pub theme: Theme,
	pub bat_theme: Option<String>,
}
