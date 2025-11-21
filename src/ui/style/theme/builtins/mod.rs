mod config;
mod loader;
mod style;

use std::sync::OnceLock;

use include_dir::{Dir, include_dir};
use loader::{BuiltinThemes, load_builtin_themes};

use crate::ui::style::theme::types::{Theme, ThemeRegistration};

const BUILTIN_THEME_DIR: Dir =
	include_dir!("$CARGO_MANIFEST_DIR/src/ui/style/theme/builtins/themes");

pub fn default_theme() -> Theme {
	builtin_themes().default_theme
}

pub(super) fn registrations() -> Vec<ThemeRegistration> {
	builtin_themes().registrations.clone()
}

fn builtin_themes() -> &'static BuiltinThemes {
	static BUILTINS: OnceLock<BuiltinThemes> = OnceLock::new();
	BUILTINS.get_or_init(|| {
		load_builtin_themes(&BUILTIN_THEME_DIR)
			.unwrap_or_else(|error| panic!("failed to load built-in themes: {error:#}"))
	})
}
