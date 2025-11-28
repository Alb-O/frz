use std::borrow::Cow;

use anyhow::{Context, Result, bail};
use include_dir::{Dir, File};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

use crate::style::theme::types::{Theme, ThemeRegistration};

pub(super) struct BuiltinThemes {
	pub(super) registrations: Vec<ThemeRegistration>,
	pub(super) default_theme: Theme,
}

#[derive(Debug, Deserialize)]
struct ThemeConfig {
	name: String,
	#[serde(default)]
	aliases: Vec<String>,
	#[serde(default)]
	default: bool,
	#[serde(default)]
	bat_theme: Option<String>,
	styles: ThemeStylesConfig,
}

impl ThemeConfig {
	fn into_document(self, context: &str) -> Result<ThemeDocument> {
		let theme = self.styles.into_theme(&format!("{context}.styles"))?;

		let mut registration = ThemeRegistration::new(self.name.clone(), theme);

		if let Some(bat_theme) = self.bat_theme {
			registration = registration.with_bat_theme(bat_theme);
		}

		let registration = self
			.aliases
			.into_iter()
			.map(|alias| alias.trim().to_string())
			.filter(|alias| !alias.is_empty())
			.fold(registration, |registration, alias| {
				registration.alias(alias)
			});

		Ok(ThemeDocument {
			registration,
			is_default: self.default,
		})
	}
}

#[derive(Debug, Deserialize)]
struct ThemeStylesConfig {
	header: StyleConfig,
	row_highlight: StyleConfig,
	prompt: StyleConfig,
	empty: StyleConfig,
	highlight: StyleConfig,
}

impl ThemeStylesConfig {
	fn into_theme(self, context: &str) -> Result<Theme> {
		Ok(Theme {
			header: self.header.to_style(&format!("{context}.header"))?,
			row_highlight: self
				.row_highlight
				.to_style(&format!("{context}.row_highlight"))?,
			prompt: self.prompt.to_style(&format!("{context}.prompt"))?,
			empty: self.empty.to_style(&format!("{context}.empty"))?,
			highlight: self.highlight.to_style(&format!("{context}.highlight"))?,
		})
	}
}

struct ThemeDocument {
	registration: ThemeRegistration,
	is_default: bool,
}

#[derive(Debug, Deserialize)]
struct StyleConfig {
	#[serde(default)]
	fg: Option<String>,
	#[serde(default)]
	bg: Option<String>,
	#[serde(default)]
	modifiers: Vec<String>,
}

impl StyleConfig {
	fn to_style(&self, context: &str) -> Result<Style> {
		let mut style = Style::new();

		if let Some(fg) = &self.fg {
			let color = parse_color(fg)
				.with_context(|| format!("{context}: invalid foreground colour `{fg}`"))?;
			style = style.fg(color);
		}

		if let Some(bg) = &self.bg {
			let color = parse_color(bg)
				.with_context(|| format!("{context}: invalid background colour `{bg}`"))?;
			style = style.bg(color);
		}

		for modifier in &self.modifiers {
			let modifier_value = parse_modifier(modifier)
				.with_context(|| format!("{context}: invalid modifier `{modifier}`"))?;
			style = style.add_modifier(modifier_value);
		}

		Ok(style)
	}
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

fn parse_color(input: &str) -> Result<Color> {
	let value = input.trim();

	if let Some(hex) = value.strip_prefix('#') {
		return parse_hex_colour(hex);
	}

	if let Some(body) = value.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
		return parse_rgb_triplet(body);
	}

	if let Some(body) = value
		.strip_prefix("ansi(")
		.and_then(|s| s.strip_suffix(')'))
	{
		let index: u8 = body
			.trim()
			.parse()
			.with_context(|| format!("invalid ANSI colour index `{body}`"))?;
		return Ok(Color::Indexed(index));
	}

	if let Ok(index) = value.parse::<u8>() {
		return Ok(Color::Indexed(index));
	}

	match normalise_key(value).as_ref() {
		"reset" | "none" | "default" => Ok(Color::Reset),
		"black" => Ok(Color::Black),
		"red" => Ok(Color::Red),
		"green" => Ok(Color::Green),
		"yellow" => Ok(Color::Yellow),
		"blue" => Ok(Color::Blue),
		"magenta" => Ok(Color::Magenta),
		"cyan" => Ok(Color::Cyan),
		"gray" | "grey" => Ok(Color::Gray),
		"dark_gray" | "dark_grey" => Ok(Color::DarkGray),
		"light_red" => Ok(Color::LightRed),
		"light_green" => Ok(Color::LightGreen),
		"light_yellow" => Ok(Color::LightYellow),
		"light_blue" => Ok(Color::LightBlue),
		"light_magenta" => Ok(Color::LightMagenta),
		"light_cyan" => Ok(Color::LightCyan),
		"white" => Ok(Color::White),
		other => bail!("unknown colour `{other}`"),
	}
}

fn parse_hex_colour(hex: &str) -> Result<Color> {
	let expanded = match hex.len() {
		3 => {
			let mut expanded = String::with_capacity(6);
			for ch in hex.chars() {
				expanded.push(ch);
				expanded.push(ch);
			}
			Cow::Owned(expanded)
		}
		6 => Cow::Borrowed(hex),
		_ => bail!("hex colours must be 3 or 6 characters long"),
	};

	let r = u8::from_str_radix(&expanded[0..2], 16)
		.with_context(|| format!("invalid red component `{hex}`"))?;
	let g = u8::from_str_radix(&expanded[2..4], 16)
		.with_context(|| format!("invalid green component `{hex}`"))?;
	let b = u8::from_str_radix(&expanded[4..6], 16)
		.with_context(|| format!("invalid blue component `{hex}`"))?;

	Ok(Color::Rgb(r, g, b))
}

fn parse_rgb_triplet(body: &str) -> Result<Color> {
	let components = body.split(',').map(|part| part.trim()).collect::<Vec<_>>();

	if components.len() != 3 {
		bail!(
			"expected three components for rgb() colour, found {}",
			components.len()
		);
	}

	let r = parse_rgb_component(components[0], 'r')?;
	let g = parse_rgb_component(components[1], 'g')?;
	let b = parse_rgb_component(components[2], 'b')?;

	Ok(Color::Rgb(r, g, b))
}

fn parse_rgb_component(value: &str, component: char) -> Result<u8> {
	value.parse::<u8>().with_context(|| {
		format!("invalid {component}-component `{value}` in rgb() colour specification")
	})
}

fn parse_modifier(input: &str) -> Result<Modifier> {
	match normalise_key(input).as_ref() {
		"bold" => Ok(Modifier::BOLD),
		"dim" => Ok(Modifier::DIM),
		"italic" => Ok(Modifier::ITALIC),
		"underline" | "underlined" => Ok(Modifier::UNDERLINED),
		"slow_blink" | "slowblink" => Ok(Modifier::SLOW_BLINK),
		"rapid_blink" | "rapidblink" | "fast_blink" => Ok(Modifier::RAPID_BLINK),
		"reversed" | "reverse" | "invert" | "inverted" => Ok(Modifier::REVERSED),
		"hidden" => Ok(Modifier::HIDDEN),
		"crossed_out" | "crossedout" | "strikethrough" => Ok(Modifier::CROSSED_OUT),
		other => bail!("unknown modifier `{other}`"),
	}
}

fn normalise_key(value: &str) -> String {
	value
		.trim()
		.to_ascii_lowercase()
		.chars()
		.map(|ch| match ch {
			'-' | ' ' => '_',
			other => other,
		})
		.collect()
}
