use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use frz::{FilesystemOptions, SearchMode};

use crate::cli::CliArgs;

use super::resolved::ResolvedConfig;
use super::ui::{apply_pane_config, parse_mode, ui_from_preset};
use super::util::{default_title_for, sanitize_extensions, sanitize_headers};

/// Mirror of the configuration file representation before CLI overrides and
/// validation are applied.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(super) struct RawConfig {
    filesystem: FilesystemSection,
    ui: UiSection,
}

/// Filesystem specific configuration options as they are read from disk.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct FilesystemSection {
    root: Option<PathBuf>,
    include_hidden: Option<bool>,
    follow_symlinks: Option<bool>,
    respect_ignore_files: Option<bool>,
    git_ignore: Option<bool>,
    git_global: Option<bool>,
    git_exclude: Option<bool>,
    threads: Option<usize>,
    max_depth: Option<usize>,
    allowed_extensions: Option<Vec<String>>,
    global_ignores: Option<Vec<String>>,
    context_label: Option<String>,
}

/// UI related configuration values prior to validation.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct UiSection {
    input_title: Option<String>,
    initial_query: Option<String>,
    theme: Option<String>,
    start_mode: Option<String>,
    preset: Option<String>,
    filter_label: Option<String>,
    detail_panel_title: Option<String>,
    facets: Option<PaneSection>,
    files: Option<PaneSection>,
    facet_headers: Option<Vec<String>>,
    file_headers: Option<Vec<String>>,
}

/// Raw configuration for a specific UI pane.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(super) struct PaneSection {
    pub(super) mode_title: Option<String>,
    pub(super) hint: Option<String>,
    pub(super) table_title: Option<String>,
    pub(super) count_label: Option<String>,
}

impl RawConfig {
    /// Apply CLI overrides on top of the raw configuration values.
    pub(super) fn apply_cli_overrides(&mut self, cli: &CliArgs) {
        if let Some(root) = cli.root.clone() {
            self.filesystem.root = Some(root);
        }
        if let Some(value) = cli.hidden {
            self.filesystem.include_hidden = Some(value);
        }
        if let Some(value) = cli.follow_symlinks {
            self.filesystem.follow_symlinks = Some(value);
        }
        if let Some(value) = cli.respect_ignore_files {
            self.filesystem.respect_ignore_files = Some(value);
        }
        if let Some(value) = cli.git_ignore {
            self.filesystem.git_ignore = Some(value);
        }
        if let Some(value) = cli.git_global {
            self.filesystem.git_global = Some(value);
        }
        if let Some(value) = cli.git_exclude {
            self.filesystem.git_exclude = Some(value);
        }
        if let Some(value) = cli.threads {
            self.filesystem.threads = Some(value);
        }
        if let Some(value) = cli.max_depth {
            self.filesystem.max_depth = Some(value);
        }
        if let Some(value) = &cli.extensions {
            self.filesystem.allowed_extensions = Some(value.clone());
        }
        if let Some(value) = &cli.global_ignores {
            self.filesystem.global_ignores = Some(value.clone());
        }
        if let Some(value) = cli.context_label.clone() {
            self.filesystem.context_label = Some(value);
        }

        if let Some(title) = cli.title.clone() {
            self.ui.input_title = Some(title);
        }
        if let Some(query) = cli.initial_query.clone() {
            self.ui.initial_query = Some(query);
        }
        if let Some(theme) = cli.theme.clone() {
            self.ui.theme = Some(theme);
        }
        if let Some(mode) = cli.start_mode {
            self.ui.start_mode = Some(mode.as_str().to_string());
        }
        if let Some(preset) = cli.ui_preset {
            self.ui.preset = Some(preset.as_str().to_string());
        }
        if let Some(label) = cli.filter_label.clone() {
            self.ui.filter_label = Some(label);
        }
        if let Some(label) = cli.detail_title.clone() {
            self.ui.detail_panel_title = Some(label);
        }
        if let Some(value) = cli.facets_mode_title.clone() {
            let facets = self.ui.facets.get_or_insert_with(PaneSection::default);
            facets.mode_title = Some(value);
        }
        if let Some(value) = cli.facets_hint.clone() {
            let facets = self.ui.facets.get_or_insert_with(PaneSection::default);
            facets.hint = Some(value);
        }
        if let Some(value) = cli.facets_table_title.clone() {
            let facets = self.ui.facets.get_or_insert_with(PaneSection::default);
            facets.table_title = Some(value);
        }
        if let Some(value) = cli.facets_count_label.clone() {
            let facets = self.ui.facets.get_or_insert_with(PaneSection::default);
            facets.count_label = Some(value);
        }
        if let Some(value) = cli.files_mode_title.clone() {
            let files = self.ui.files.get_or_insert_with(PaneSection::default);
            files.mode_title = Some(value);
        }
        if let Some(value) = cli.files_hint.clone() {
            let files = self.ui.files.get_or_insert_with(PaneSection::default);
            files.hint = Some(value);
        }
        if let Some(value) = cli.files_table_title.clone() {
            let files = self.ui.files.get_or_insert_with(PaneSection::default);
            files.table_title = Some(value);
        }
        if let Some(value) = cli.files_count_label.clone() {
            let files = self.ui.files.get_or_insert_with(PaneSection::default);
            files.count_label = Some(value);
        }
        if let Some(headers) = &cli.facet_headers {
            self.ui.facet_headers = Some(headers.clone());
        }
        if let Some(headers) = &cli.file_headers {
            self.ui.file_headers = Some(headers.clone());
        }
    }

    /// Convert the raw configuration into a [`ResolvedConfig`], validating and
    /// filling defaults where required.
    pub(super) fn resolve(self) -> Result<ResolvedConfig> {
        let mut root = match self.filesystem.root {
            Some(path) => path,
            None => env::current_dir().context("failed to determine working directory")?,
        };
        if root.is_relative() {
            root = env::current_dir()
                .context("failed to resolve current directory for root")?
                .join(root);
        }
        root = fs::canonicalize(&root).with_context(|| {
            format!("failed to canonicalize filesystem root {}", root.display())
        })?;

        let metadata = fs::metadata(&root)
            .with_context(|| format!("failed to inspect filesystem root {}", root.display()))?;
        ensure!(metadata.is_dir(), "filesystem root must be a directory");

        let mut filesystem = FilesystemOptions::default();
        filesystem.include_hidden = self.filesystem.include_hidden.unwrap_or(true);
        filesystem.follow_symlinks = self.filesystem.follow_symlinks.unwrap_or(false);
        filesystem.respect_ignore_files = self.filesystem.respect_ignore_files.unwrap_or(true);
        filesystem.git_ignore = self.filesystem.git_ignore.unwrap_or(true);
        filesystem.git_global = self.filesystem.git_global.unwrap_or(true);
        filesystem.git_exclude = self.filesystem.git_exclude.unwrap_or(true);
        filesystem.threads = self.filesystem.threads;
        filesystem.max_depth = self.filesystem.max_depth;
        filesystem.allowed_extensions = self
            .filesystem
            .allowed_extensions
            .map(sanitize_extensions)
            .filter(|exts| !exts.is_empty());
        filesystem.context_label = self.filesystem.context_label.clone();
        if let Some(ignores) = self.filesystem.global_ignores.clone() {
            filesystem.global_ignores = ignores;
        }

        let mut ui = ui_from_preset(self.ui.preset.as_deref())?;
        if let Some(label) = self.ui.filter_label {
            ui.filter_label = label;
        }
        if let Some(detail) = self.ui.detail_panel_title {
            ui.detail_panel_title = detail;
        }
        if let Some(pane) = self.ui.facets {
            if let Some(target) = ui.pane_mut(SearchMode::FACETS) {
                apply_pane_config(target, pane);
            }
        }
        if let Some(pane) = self.ui.files {
            if let Some(target) = ui.pane_mut(SearchMode::FILES) {
                apply_pane_config(target, pane);
            }
        }

        let default_title = filesystem
            .context_label
            .clone()
            .unwrap_or_else(|| default_title_for(&root));

        let facet_headers = self
            .ui
            .facet_headers
            .map(sanitize_headers)
            .filter(|headers| !headers.is_empty());
        let file_headers = self
            .ui
            .file_headers
            .map(sanitize_headers)
            .filter(|headers| !headers.is_empty());

        let input_title = match self.ui.input_title {
            Some(title) => Some(title),
            None => Some(default_title),
        };
        let initial_query = self.ui.initial_query.unwrap_or_default();
        let theme = self.ui.theme;
        let start_mode = match self.ui.start_mode {
            Some(mode) => Some(parse_mode(&mode)?),
            None => None,
        };

        Ok(ResolvedConfig {
            root,
            filesystem,
            input_title,
            initial_query,
            theme,
            start_mode,
            ui,
            facet_headers,
            file_headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_overrides_take_precedence() {
        let mut cli = CliArgs::parse_from(["frz", "--start-mode", "files"]);
        cli.root = Some(PathBuf::from("/tmp"));
        cli.hidden = Some(false);
        cli.follow_symlinks = Some(true);
        cli.respect_ignore_files = Some(false);
        cli.git_ignore = Some(false);
        cli.git_global = Some(false);
        cli.git_exclude = Some(false);
        cli.threads = Some(4);
        cli.max_depth = Some(10);
        cli.extensions = Some(vec!["rs".into()]);
        cli.global_ignores = Some(vec!["target".into()]);
        cli.context_label = Some("ctx".into());
        cli.title = Some("title".into());
        cli.initial_query = Some("query".into());
        cli.theme = Some("dark".into());
        cli.filter_label = Some("filter".into());
        cli.detail_title = Some("detail".into());
        cli.facets_mode_title = Some("facet".into());
        cli.facets_hint = Some("hint".into());
        cli.facets_table_title = Some("table".into());
        cli.facets_count_label = Some("count".into());
        cli.files_mode_title = Some("file".into());
        cli.files_hint = Some("file hint".into());
        cli.files_table_title = Some("files table".into());
        cli.files_count_label = Some("files count".into());
        cli.facet_headers = Some(vec!["a".into()]);
        cli.file_headers = Some(vec!["b".into()]);

        let mut config = RawConfig::default();
        config.apply_cli_overrides(&cli);

        assert_eq!(config.filesystem.root, cli.root);
        assert_eq!(config.ui.input_title, cli.title);
        assert_eq!(config.ui.initial_query, cli.initial_query);
        assert_eq!(config.ui.theme, cli.theme);
        assert_eq!(config.ui.start_mode, Some("files".into()));
        assert_eq!(config.ui.facet_headers, cli.facet_headers);
        assert_eq!(config.ui.file_headers, cli.file_headers);
    }
}
