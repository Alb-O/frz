use std::env;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use config::{Config, ConfigError, File};

use crate::cli::CliArgs;
use frz::app_dirs;

/// Build a [`Config`] instance by combining default locations with CLI overrides.
pub(super) fn build_config(cli: &CliArgs) -> Result<Config> {
    let mut builder = Config::builder();

    if !cli.no_config {
        for path in default_config_files() {
            builder = builder.add_source(File::from(path).required(false));
        }
    }

    for path in &cli.config {
        builder = builder.add_source(File::from(path.clone()).required(true));
    }

    builder = builder.add_source(
        config::Environment::with_prefix("frz")
            .separator("__")
            .try_parsing(true)
            .list_separator(","),
    );

    builder.build().map_err(|err| match err {
        ConfigError::Frozen => anyhow!("configuration builder is frozen"),
        other => other.into(),
    })
}

/// Discover the default configuration file locations that should be consulted.
pub(super) fn default_config_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(dir) = app_dirs::get_config_dir() {
        files.push(dir.join("config.toml"));
    }

    if let Ok(current_dir) = env::current_dir() {
        files.push(current_dir.join(".frz.toml"));
        files.push(current_dir.join("frz.toml"));
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_files_include_current_directory_variants() {
        let files = default_config_files();
        assert!(files.iter().any(|path| path.ends_with(".frz.toml")));
        assert!(files.iter().any(|path| path.ends_with("frz.toml")));
    }
}
