//! Visual styling utilities.
//!
//! The `style` module is the umbrella for UI appearance. Themes represent the
//! color schemes applied to the terminal UI, while additional styling options
//! can be layered alongside themes in the future.

pub mod theme;

pub use theme::{
	AliasConflict, Theme, ThemeDefinition, ThemeDescriptor, ThemeRegistration,
	ThemeRegistrationReport, bat_theme, builtin_themes, by_name, default_theme, descriptors, names,
	register_additional, register_definitions,
};

/// Aggregate container for styling knobs. Additional visual tweaks can be
/// surfaced here over time while keeping themes focused on color schemes.
#[derive(Clone, Debug, Default)]
pub struct StyleConfig {
	pub theme: Theme,
}

impl StyleConfig {
	#[must_use]
	pub fn with_theme(theme: Theme) -> Self {
		Self { theme }
	}
}
