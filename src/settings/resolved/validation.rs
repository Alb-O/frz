use super::{ConfigError, ConfigSources, ResolvedConfig};

pub(super) fn validate(
	config: &ResolvedConfig,
	sources: &ConfigSources,
) -> Result<(), ConfigError> {
	if let Some(threads) = config.filesystem.threads
		&& threads == 0
	{
		return Err(ConfigError::invalid(
			"filesystem.threads",
			threads.to_string(),
			sources.source_for_threads(),
			"must be greater than zero",
		));
	}

	if let Some(max_depth) = config.filesystem.max_depth
		&& max_depth == 0
	{
		return Err(ConfigError::invalid(
			"filesystem.max_depth",
			max_depth.to_string(),
			sources.source_for_max_depth(),
			"must be at least 1",
		));
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use frz::{FilesystemOptions, UiConfig};

	use super::super::SettingSource;
	use super::*;

	#[test]
	fn validation_rejects_zero_threads() {
		let filesystem = FilesystemOptions {
			threads: Some(0),
			..FilesystemOptions::default()
		};
		let config = ResolvedConfig {
			root: PathBuf::from("/tmp"),
			filesystem,
			input_title: None,
			initial_query: String::new(),
			theme: None,
			ui: UiConfig::default(),
			file_headers: None,
		};

		let sources = ConfigSources {
			filesystem_threads: Some(SettingSource::CliFlag("--threads")),
			..ConfigSources::default()
		};

		let err = validate(&config, &sources).unwrap_err();
		assert!(matches!(err.key, "filesystem.threads"));
		let message = err.to_string();
		assert!(message.contains("value: 0"));
		assert!(message.contains("CLI flag"));
	}

	#[test]
	fn validation_rejects_zero_max_depth() {
		let filesystem = FilesystemOptions {
			max_depth: Some(0),
			..FilesystemOptions::default()
		};
		let config = ResolvedConfig {
			root: PathBuf::from("/tmp"),
			filesystem,
			input_title: None,
			initial_query: String::new(),
			theme: None,
			ui: UiConfig::default(),
			file_headers: None,
		};

		let sources = ConfigSources {
			filesystem_max_depth: Some(SettingSource::Environment("FRZ__FILESYSTEM__MAX_DEPTH")),
			..ConfigSources::default()
		};

		let err = validate(&config, &sources).unwrap_err();
		assert!(matches!(err.key, "filesystem.max_depth"));
		let message = err.to_string();
		assert!(message.contains("value: 0"));
		assert!(message.contains("environment variable"));
	}
}
