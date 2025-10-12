use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail, ensure};
use config::{Config, ConfigError, File};
use serde::Deserialize;

use frz::{FilesystemOptions, PaneUiConfig, SearchMode, UiConfig};

use crate::cli::CliArgs;
use frz::app_dirs;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    filesystem: FilesystemSection,
    ui: UiSection,
}

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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct PaneSection {
    mode_title: Option<String>,
    hint: Option<String>,
    table_title: Option<String>,
    count_label: Option<String>,
}

pub struct ResolvedConfig {
    pub root: PathBuf,
    pub filesystem: FilesystemOptions,
    pub input_title: Option<String>,
    pub initial_query: String,
    pub theme: Option<String>,
    pub start_mode: Option<SearchMode>,
    pub ui: UiConfig,
    pub facet_headers: Option<Vec<String>>,
    pub file_headers: Option<Vec<String>>,
}

impl ResolvedConfig {
    pub fn print_summary(&self) {
        println!("Effective configuration:");
        println!("  Root: {}", self.root.display());
        println!(
            "  Include hidden: {}",
            bool_to_word(self.filesystem.include_hidden)
        );
        println!(
            "  Follow symlinks: {}",
            bool_to_word(self.filesystem.follow_symlinks)
        );
        println!(
            "  Respect ignore files: {}",
            bool_to_word(self.filesystem.respect_ignore_files)
        );
        println!("  Git ignore: {}", bool_to_word(self.filesystem.git_ignore));
        println!("  Git global: {}", bool_to_word(self.filesystem.git_global));
        println!(
            "  Git exclude: {}",
            bool_to_word(self.filesystem.git_exclude)
        );
        match self.filesystem.max_depth {
            Some(depth) => println!("  Max depth: {depth}"),
            None => println!("  Max depth: unlimited"),
        }
        match &self.filesystem.allowed_extensions {
            Some(exts) if !exts.is_empty() => {
                println!("  Allowed extensions: {}", exts.join(", "));
            }
            _ => println!("  Allowed extensions: (all)"),
        }
        if let Some(threads) = self.filesystem.threads {
            println!("  Threads: {threads}");
        }
        if let Some(label) = &self.filesystem.context_label {
            println!("  Context label: {label}");
        }
        if !self.filesystem.global_ignores.is_empty() {
            println!(
                "  Global ignores: {}",
                self.filesystem.global_ignores.join(", ")
            );
        }
        println!(
            "  UI theme: {}",
            self.theme.as_deref().unwrap_or("(use the library default)")
        );
        println!(
            "  Start mode: {}",
            self.start_mode
                .map(|mode| match mode {
                    SearchMode::Facets => "facets".to_string(),
                    SearchMode::Files => "files".to_string(),
                })
                .unwrap_or_else(|| "(auto)".to_string())
        );
        if let Some(title) = &self.input_title {
            println!("  Prompt title: {title}");
        }
        if !self.initial_query.is_empty() {
            println!("  Initial query: {}", self.initial_query);
        }
        if let Some(headers) = &self.facet_headers {
            println!("  Facet headers: {}", headers.join(", "));
        }
        if let Some(headers) = &self.file_headers {
            println!("  File headers: {}", headers.join(", "));
        }
    }
}

pub fn load(cli: &CliArgs) -> Result<ResolvedConfig> {
    let builder = build_config(cli)?;
    let mut raw: RawConfig = builder
        .try_deserialize()
        .map_err(|err| anyhow!("failed to deserialize configuration: {err}"))?;
    raw.apply_cli_overrides(cli);
    raw.resolve()
}

fn build_config(cli: &CliArgs) -> Result<Config> {
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

fn default_config_files() -> Vec<PathBuf> {
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

impl RawConfig {
    fn apply_cli_overrides(&mut self, cli: &CliArgs) {
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

    fn resolve(self) -> Result<ResolvedConfig> {
        let mut root = match self.filesystem.root {
            Some(path) => path,
            None => env::current_dir().context("failed to determine working directory")?,
        };
        if root.is_relative() {
            root = env::current_dir()
                .context("failed to resolve current directory for root")?
                .join(root);
        }
        // Canonicalize the path to remove any dot components (../, ./) and
        // resolve symlinks. We canonicalize before further checks so the
        // stored `root` and any UI titles are based on the resolved path.
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
            apply_pane_config(&mut ui.facets, pane);
        }
        if let Some(pane) = self.ui.files {
            apply_pane_config(&mut ui.files, pane);
        }

        let default_title = filesystem
            .context_label
            .clone()
            .unwrap_or_else(|| default_title_for(&root));

        let facet_headers = self
            .ui
            .facet_headers
            .map(|headers| sanitize_headers(headers).collect::<Vec<_>>())
            .filter(|headers| !headers.is_empty());
        let file_headers = self
            .ui
            .file_headers
            .map(|headers| sanitize_headers(headers).collect::<Vec<_>>())
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

fn ui_from_preset(preset: Option<&str>) -> Result<UiConfig> {
    let Some(raw) = preset else {
        return Ok(UiConfig::default());
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(UiConfig::default());
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "default" => Ok(UiConfig::default()),
        "tags-and-files" | "tags_and_files" | "tags" => Ok(UiConfig::tags_and_files()),
        other => bail!("unknown UI preset '{other}'"),
    }
}

fn apply_pane_config(target: &mut PaneUiConfig, pane: PaneSection) {
    if let Some(value) = pane.mode_title {
        target.mode_title = value;
    }
    if let Some(value) = pane.hint {
        target.hint = value;
    }
    if let Some(value) = pane.table_title {
        target.table_title = value;
    }
    if let Some(value) = pane.count_label {
        target.count_label = value;
    }
}

fn parse_mode(value: &str) -> Result<SearchMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "facets" => Ok(SearchMode::Facets),
        "files" => Ok(SearchMode::Files),
        other => bail!("unknown start mode '{other}'"),
    }
}

fn sanitize_extensions(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();
    for value in values {
        let normalized = value.trim().trim_start_matches('.').to_ascii_lowercase();
        if normalized.is_empty() {
            continue;
        }
        if seen.insert(normalized.clone()) {
            cleaned.push(normalized);
        }
    }
    cleaned
}

fn sanitize_headers(headers: Vec<String>) -> impl Iterator<Item = String> {
    headers
        .into_iter()
        .map(|header| header.trim().to_string())
        .filter(|header| !header.is_empty())
}

fn default_title_for(root: &Path) -> String {
    // Prefer showing the resolved path relative to $HOME (e.g. ~/projects/foo)
    // if applicable. Always return a cleaned path (no dot components) since
    // `root` is canonicalized in `resolve()`.
    fn shorten(path: &Path) -> String {
        if let Some(home_os) = env::var_os("HOME") {
            let home = PathBuf::from(home_os);
            if let Ok(rel) = path.strip_prefix(&home) {
                // If path == home -> show ~
                if rel.components().next().is_none() {
                    return "~".to_string();
                }
                let sep = std::path::MAIN_SEPARATOR;
                return format!("~{}{}", sep, rel.display());
            }
        }
        path.display().to_string()
    }

    shorten(root)
}

fn bool_to_word(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_shorten_home() {
        // Ensure HOME is set for the test
        let home = env::var("HOME").expect("HOME must be set for this test");
        let mut path = PathBuf::from(home);
        path.push("some_subdir");
        let displayed = default_title_for(&path);
        assert!(displayed.starts_with("~"));
    }

    #[test]
    fn test_canonicalize_removes_dots() {
        let dir = tempdir().unwrap();
        let base = dir.path().to_path_buf();
        // Create a nested path and a ../ reference
        let nested = base.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();
        let dotted = nested.join("..").join("b");
        // canonicalize as settings.resolve() would do
        let canon = std::fs::canonicalize(&dotted).unwrap();
        // canonical path should equal nested
        assert_eq!(canon, nested.canonicalize().unwrap());
    }
}
