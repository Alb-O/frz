//! Visual styling utilities.
//!
//! The `style` module is the umbrella for UI appearance. Themes represent the
//! color schemes applied to the terminal UI, while additional styling options
//! can be layered alongside themes in the future.

/// The `theme` submodule contains definitions, built-in themes, and
/// theme registration utilities.
pub mod theme;

/// Re-export theme types and utilities.
pub use theme::{
	AliasConflict, Theme, ThemeDescriptor, ThemeRegistration, ThemeRegistrationReport, bat_theme,
	builtin_themes, by_name, default_theme, descriptors, names, register_additional,
};

/// Aggregate container for styling knobs. Additional visual tweaks can be
/// surfaced here over time while keeping themes focused on color schemes.
#[derive(Clone, Debug, Default)]
pub struct StyleConfig {
	/// The active theme for the UI.
	pub theme: Theme,
}

impl StyleConfig {
	/// Creates a new style configuration with the given theme.
	#[must_use]
	pub fn with_theme(theme: Theme) -> Self {
		Self { theme }
	}
}
