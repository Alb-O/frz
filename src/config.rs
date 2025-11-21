use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result, ensure};
use frz::{FilesystemOptions, UiConfig};

use crate::cli::CliArgs;

/// Simple application configuration derived from CLI arguments and defaults.
#[derive(Debug)]
pub struct Config {
	pub root: PathBuf,
	pub filesystem: FilesystemOptions,
	pub input_title: Option<String>,
	pub initial_query: String,
	pub theme: Option<String>,
	pub ui: UiConfig,
	pub file_headers: Option<Vec<String>>,
}

impl Config {
	/// Build configuration from CLI arguments with sensible defaults.
	pub fn from_cli(cli: &CliArgs) -> Result<Self> {
		let root = resolve_root(cli)?;
		let filesystem = build_filesystem_options(cli);
		let context_label = filesystem.context_label.clone();

		let input_title = cli
			.title
			.clone()
			.or(context_label)
			.or_else(|| Some(default_title_for(&root)));

		let initial_query = cli.initial_query.clone().unwrap_or_default();
		let theme = cli.theme.clone();
		let ui = build_ui_config(cli)?;
		let file_headers = cli
			.file_headers
			.as_ref()
			.map(|headers| sanitize_headers(headers.clone()));

		// Validate
		if let Some(threads) = filesystem.threads {
			ensure!(threads > 0, "threads must be greater than zero");
		}
		if let Some(max_depth) = filesystem.max_depth {
			ensure!(max_depth > 0, "max-depth must be at least 1");
		}

		Ok(Self {
			root,
			filesystem,
			input_title,
			initial_query,
			theme,
			ui,
			file_headers,
		})
	}
}

fn resolve_root(cli: &CliArgs) -> Result<PathBuf> {
	let mut root = match &cli.root {
		Some(path) => path.clone(),
		None => env::current_dir().context("failed to determine working directory")?,
	};

	if root.is_relative() {
		root = env::current_dir()
			.context("failed to resolve current directory for root")?
			.join(root);
	}

	root = fs::canonicalize(&root)
		.with_context(|| format!("failed to canonicalize filesystem root {}", root.display()))?;

	let metadata = fs::metadata(&root)
		.with_context(|| format!("failed to inspect filesystem root {}", root.display()))?;
	ensure!(metadata.is_dir(), "filesystem root must be a directory");

	Ok(root)
}

fn build_filesystem_options(cli: &CliArgs) -> FilesystemOptions {
	let allowed_extensions = cli
		.extensions
		.as_ref()
		.map(|exts| sanitize_extensions(exts.clone()))
		.filter(|exts| !exts.is_empty());

	FilesystemOptions {
		include_hidden: cli.hidden.unwrap_or(true),
		follow_symlinks: cli.follow_symlinks.unwrap_or(false),
		respect_ignore_files: cli.respect_ignore_files.unwrap_or(true),
		git_ignore: cli.git_ignore.unwrap_or(true),
		git_global: cli.git_global.unwrap_or(true),
		git_exclude: cli.git_exclude.unwrap_or(true),
		threads: cli.threads,
		max_depth: cli.max_depth,
		allowed_extensions,
		context_label: cli.context_label.clone(),
		global_ignores: cli.global_ignores.clone().unwrap_or_default(),
	}
}

fn build_ui_config(cli: &CliArgs) -> Result<UiConfig> {
	use frz::extensions::builtin::files;

	let preset = cli.ui_preset.as_ref().map(|p| p.as_str());
	let mut ui = ui_from_preset(preset)?;

	if let Some(label) = &cli.filter_label {
		ui.filter_label = label.clone();
	}
	if let Some(detail) = &cli.detail_title {
		ui.detail_panel_title = detail.clone();
	}

	// Apply files pane overrides
	if (cli.files_mode_title.is_some()
		|| cli.files_hint.is_some()
		|| cli.files_table_title.is_some()
		|| cli.files_count_label.is_some())
		&& let Some(mode) = ui.mode_by_id(files::DATASET_KEY)
		&& let Some(pane) = ui.pane_mut(mode)
	{
		if let Some(title) = &cli.files_mode_title {
			pane.mode_title = title.clone();
		}
		if let Some(hint) = &cli.files_hint {
			pane.hint = hint.clone();
		}
		if let Some(title) = &cli.files_table_title {
			pane.table_title = title.clone();
		}
		if let Some(label) = &cli.files_count_label {
			pane.count_label = label.clone();
		}
	}

	Ok(ui)
}

fn ui_from_preset(preset: Option<&str>) -> Result<UiConfig> {
	match preset {
		None | Some("default") => Ok(UiConfig::default()),
		Some(name) => anyhow::bail!("unknown UI preset: {}", name),
	}
}

fn default_title_for(root: &Path) -> String {
	root.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("frz")
		.to_string()
}

fn sanitize_extensions(exts: Vec<String>) -> Vec<String> {
	exts.into_iter()
		.map(|ext| {
			let trimmed = ext.trim();
			trimmed.strip_prefix('.').unwrap_or(trimmed).to_string()
		})
		.filter(|ext| !ext.is_empty())
		.collect()
}

fn sanitize_headers(headers: Vec<String>) -> Vec<String> {
	headers
		.into_iter()
		.map(|h| h.trim().to_string())
		.filter(|h| !h.is_empty())
		.collect()
}
