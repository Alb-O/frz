use anyhow::{Context, Result, bail};
use include_dir::{Dir, File};

use super::config::{ThemeConfig, ThemeDocument};
use crate::features::tui_app::style::theme::types::{Theme, ThemeRegistration};

pub(super) struct BuiltinThemes {
	pub(super) registrations: Vec<ThemeRegistration>,
	pub(super) default_theme: Theme,
}

pub(super) fn load_builtin_themes(dir: &Dir) -> Result<BuiltinThemes> {
	let mut registrations = Vec::new();
	let mut default_theme: Option<(Theme, String)> = None;

	let mut files: Vec<_> = dir.files().collect();
	files.sort_by(|a, b| a.path().cmp(b.path()));

	for file in files {
		let document = parse_theme_document(file)?;
		let theme = document.registration.theme;

		if document.is_default {
			if let Some((_, existing_name)) = &default_theme {
				bail!(
					"multiple built-in themes are marked as default (`{existing_name}` and `{}`)",
					document.registration.name
				);
			}

			default_theme = Some((theme, document.registration.name.clone()));
		}

		registrations.push(document.registration);
	}

	if registrations.is_empty() {
		bail!("no built-in theme definitions were found");
	}

	let default_theme = default_theme
		.map(|(theme, _)| theme)
		.or_else(|| registrations.first().map(|registration| registration.theme))
		.expect("at least one registration exists");

	Ok(BuiltinThemes {
		registrations,
		default_theme,
	})
}

fn parse_theme_document(file: &File) -> Result<ThemeDocument> {
	let path = file.path();
	let contents = file
		.contents_utf8()
		.with_context(|| format!("{path:?} is not valid UTF-8"))?;

	let config: ThemeConfig = toml::from_str(contents)
		.with_context(|| format!("failed to parse built-in theme definition in {path:?}"))?;

	config.into_document(&format!("{path:?}"))
}
