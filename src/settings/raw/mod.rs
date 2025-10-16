use anyhow::{Error, Result};
use serde::Deserialize;
use std::env;

use crate::cli::CliArgs;

use super::resolved::{ConfigSources, ResolvedConfig, SettingSource};
use super::util::default_title_for;

mod filesystem;
mod ui;

use filesystem::FilesystemSection;
use ui::UiSection;

pub(super) use ui::PaneSection;

/// Mirror of the configuration file representation before CLI overrides and
/// validation are applied.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(super) struct RawConfig {
    filesystem: FilesystemSection,
    ui: UiSection,
}

impl RawConfig {
    /// Apply CLI overrides on top of the raw configuration values.
    pub(super) fn apply_cli_overrides(&mut self, cli: &CliArgs) {
        self.filesystem.apply_cli_overrides(cli);
        self.ui.apply_cli_overrides(cli);
    }

    /// Convert the raw configuration into a [`ResolvedConfig`], validating and
    /// filling defaults where required.
    pub(super) fn resolve(self, cli: &CliArgs) -> Result<ResolvedConfig> {
        let sources = ConfigSources {
            filesystem_threads: detect_source(
                cli.threads.is_some(),
                self.filesystem.threads.is_some(),
                "FRZ__FILESYSTEM__THREADS",
                "--threads",
                "filesystem.threads",
            ),
            filesystem_max_depth: detect_source(
                cli.max_depth.is_some(),
                self.filesystem.max_depth.is_some(),
                "FRZ__FILESYSTEM__MAX_DEPTH",
                "--max-depth",
                "filesystem.max_depth",
            ),
        };

        let (root, filesystem) = self.filesystem.resolve()?;
        let default_title = filesystem
            .context_label
            .clone()
            .unwrap_or_else(|| default_title_for(&root));

        let ui = self.ui.finalize(default_title)?;

        let config = ResolvedConfig {
            root,
            filesystem,
            input_title: ui.input_title,
            initial_query: ui.initial_query,
            theme: ui.theme,
            start_mode: ui.start_mode,
            ui: ui.ui,
            facet_headers: ui.facet_headers,
            file_headers: ui.file_headers,
        };

        config.validate(&sources).map_err(Error::new)?;

        Ok(config)
    }
}

fn detect_source(
    cli_present: bool,
    value_present: bool,
    env_var: &'static str,
    cli_flag: &'static str,
    key: &'static str,
) -> Option<SettingSource> {
    if !value_present {
        return None;
    }

    if cli_present {
        return Some(SettingSource::CliFlag(cli_flag));
    }

    if env::var_os(env_var).is_some() {
        return Some(SettingSource::Environment(env_var));
    }

    Some(SettingSource::ConfigKey(key))
}

#[cfg(test)]
mod tests;
