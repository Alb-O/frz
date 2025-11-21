use std::fmt::Write;

use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use frz::app_dirs;

/// Produce the full version banner including config and data directories.
pub(super) fn long_version() -> &'static str {
	let config_dir = match app_dirs::get_config_dir() {
		Ok(path) => path.display().to_string(),
		Err(err) => format!("unavailable ({err})"),
	};
	let data_dir = match app_dirs::get_data_dir() {
		Ok(path) => path.display().to_string(),
		Err(err) => format!("unavailable ({err})"),
	};

	let mut details = format!("frz {}", env!("CARGO_PKG_VERSION"));
	let _ = writeln!(details);
	let _ = writeln!(details, "config directory: {config_dir}");
	let _ = writeln!(details, "data directory: {data_dir}");

	Box::leak(details.into_boxed_str())
}

/// Create the clap styles used for custom colour output.
pub(super) fn cli_styles() -> Styles {
	Styles::styled()
		.header(AnsiColor::Green.on_default().effects(Effects::BOLD))
		.usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
		.literal(AnsiColor::Cyan.on_default())
		.placeholder(AnsiColor::Yellow.on_default())
}
