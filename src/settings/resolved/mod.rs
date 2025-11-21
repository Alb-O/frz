use std::path::PathBuf;

use frz::{FilesystemOptions, UiConfig};

mod errors;
mod sources;
mod summary;
mod validation;

pub(crate) use errors::ConfigError;
pub(crate) use sources::{ConfigSources, SettingSource};

/// Application-ready configuration derived from user input, config files and
/// sensible defaults.
#[derive(Debug)]
pub struct ResolvedConfig {
	pub root: PathBuf,
	pub filesystem: FilesystemOptions,
	pub input_title: Option<String>,
	pub initial_query: String,
	pub theme: Option<String>,
	pub ui: UiConfig,
	pub file_headers: Option<Vec<String>>,
}

impl ResolvedConfig {
	pub(super) fn validate(&self, sources: &ConfigSources) -> Result<(), ConfigError> {
		validation::validate(self, sources)
	}

	/// Print a human readable summary of the effective configuration.
	pub fn print_summary(&self) {
		summary::print_summary(self);
	}
}
