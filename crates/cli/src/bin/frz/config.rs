use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result, ensure};
use frz_core::filesystem_indexer::FilesystemOptions;
use frz_tui::UiLabels;

use crate::cli::CliArgs;

/// Simple application configuration derived from CLI arguments and defaults.
#[derive(Debug)]
pub struct Config {
	pub root: PathBuf,
	pub filesystem: FilesystemOptions,
	pub initial_query: String,
	pub theme: Option<String>,
	pub ui: UiLabels,
	pub file_headers: Option<Vec<String>>,
}

impl Config {
	/// Build configuration from CLI arguments with sensible defaults.
	pub fn from_cli(cli: &CliArgs) -> Result<Self> {
		let root = resolve_root(cli)?;
		let filesystem = build_filesystem_options(cli);

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
			initial_query,
			theme,
			ui,
			file_headers,
		})
	}
}

/// Resolve the filesystem root directory from CLI args, validating it exists and is a directory.
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

/// Construct filesystem scanning options from CLI arguments with appropriate defaults.
fn build_filesystem_options(cli: &CliArgs) -> FilesystemOptions {
	let allowed_extensions = cli
		.extensions
		.as_ref()
		.map(|exts| sanitize_extensions(exts.clone()))
		.filter(|exts| !exts.is_empty());

	let mut options = FilesystemOptions::default();

	options.include_hidden = cli.hidden.unwrap_or(options.include_hidden);
	options.follow_symlinks = cli.follow_symlinks.unwrap_or(options.follow_symlinks);
	options.respect_ignore_files = cli
		.respect_ignore_files
		.unwrap_or(options.respect_ignore_files);
	options.git_ignore = cli.git_ignore.unwrap_or(options.git_ignore);
	options.git_global = cli.git_global.unwrap_or(options.git_global);
	options.git_exclude = cli.git_exclude.unwrap_or(options.git_exclude);
	options.threads = cli.threads;
	options.max_depth = cli.max_depth;
	options.allowed_extensions = allowed_extensions;
	options.context_label = cli.context_label.clone();

	if let Some(extra_ignores) = cli.global_ignores.as_ref() {
		for ignore in extra_ignores {
			if !options.global_ignores.contains(ignore) {
				options.global_ignores.push(ignore.clone());
			}
		}
	}

	options
}

/// Build UI configuration from CLI arguments, applying preset and overrides.
fn build_ui_config(cli: &CliArgs) -> Result<UiLabels> {
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
		&& let Some(pane) = ui.pane_mut()
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

/// Resolve a UI preset name to its configuration.
fn ui_from_preset(preset: Option<&str>) -> Result<UiLabels> {
	match preset {
		None | Some("default") => Ok(UiLabels::default()),
		Some(name) => anyhow::bail!("unknown UI preset: {}", name),
	}
}

/// Normalize file extensions by removing leading dots and filtering empty values.
fn sanitize_extensions(exts: Vec<String>) -> Vec<String> {
	exts.into_iter()
		.map(|ext| {
			let trimmed = ext.trim();
			trimmed.strip_prefix('.').unwrap_or(trimmed).to_string()
		})
		.filter(|ext| !ext.is_empty())
		.collect()
}

/// Trim and filter empty header strings.
fn sanitize_headers(headers: Vec<String>) -> Vec<String> {
	headers
		.into_iter()
		.map(|h| h.trim().to_string())
		.filter(|h| !h.is_empty())
		.collect()
}
