use anyhow::Result;
use serde::Deserialize;

use crate::cli::CliArgs;

use super::resolved::ResolvedConfig;
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
    pub(super) fn resolve(self) -> Result<ResolvedConfig> {
        let (root, filesystem) = self.filesystem.resolve()?;
        let default_title = filesystem
            .context_label
            .clone()
            .unwrap_or_else(|| default_title_for(&root));

        let ui = self.ui.finalize(default_title)?;

        Ok(ResolvedConfig {
            root,
            filesystem,
            input_title: ui.input_title,
            initial_query: ui.initial_query,
            theme: ui.theme,
            start_mode: ui.start_mode,
            ui: ui.ui,
            facet_headers: ui.facet_headers,
            file_headers: ui.file_headers,
        })
    }
}

#[cfg(test)]
mod tests;
