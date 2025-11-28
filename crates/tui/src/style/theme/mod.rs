mod builtins;
mod registry;
mod types;

pub use builtins::default_theme;
pub use registry::{bat_theme, by_name, descriptors, names, register_additional};
pub use types::{
	AliasConflict, Theme, ThemeDescriptor, ThemeRegistration, ThemeRegistrationReport,
};

/// Return the built-in themes bundled with the application.
#[must_use]
pub fn builtin_themes() -> Vec<ThemeRegistration> {
	builtins::registrations()
}

impl Default for Theme {
	fn default() -> Self {
		default_theme()
	}
}
