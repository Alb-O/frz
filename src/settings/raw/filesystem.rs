use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use frz::FilesystemOptions;

use crate::cli::CliArgs;

use super::super::util::sanitize_extensions;

/// Filesystem specific configuration options as they are read from disk.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(super) struct FilesystemSection {
    pub(super) root: Option<PathBuf>,
    pub(super) include_hidden: Option<bool>,
    pub(super) follow_symlinks: Option<bool>,
    pub(super) respect_ignore_files: Option<bool>,
    pub(super) git_ignore: Option<bool>,
    pub(super) git_global: Option<bool>,
    pub(super) git_exclude: Option<bool>,
    pub(super) threads: Option<usize>,
    pub(super) max_depth: Option<usize>,
    pub(super) allowed_extensions: Option<Vec<String>>,
    pub(super) global_ignores: Option<Vec<String>>,
    pub(super) context_label: Option<String>,
}

impl FilesystemSection {
    pub(super) fn apply_cli_overrides(&mut self, cli: &CliArgs) {
        if let Some(root) = cli.root.clone() {
            self.root = Some(root);
        }
        if let Some(value) = cli.hidden {
            self.include_hidden = Some(value);
        }
        if let Some(value) = cli.follow_symlinks {
            self.follow_symlinks = Some(value);
        }
        if let Some(value) = cli.respect_ignore_files {
            self.respect_ignore_files = Some(value);
        }
        if let Some(value) = cli.git_ignore {
            self.git_ignore = Some(value);
        }
        if let Some(value) = cli.git_global {
            self.git_global = Some(value);
        }
        if let Some(value) = cli.git_exclude {
            self.git_exclude = Some(value);
        }
        if let Some(value) = cli.threads {
            self.threads = Some(value);
        }
        if let Some(value) = cli.max_depth {
            self.max_depth = Some(value);
        }
        if let Some(value) = &cli.extensions {
            self.allowed_extensions = Some(value.clone());
        }
        if let Some(value) = &cli.global_ignores {
            self.global_ignores = Some(value.clone());
        }
        if let Some(value) = cli.context_label.clone() {
            self.context_label = Some(value);
        }
    }

    pub(super) fn resolve(self) -> Result<(PathBuf, FilesystemOptions)> {
        let mut root = match self.root {
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

        let filesystem = FilesystemOptions {
            include_hidden: self.include_hidden.unwrap_or(true),
            follow_symlinks: self.follow_symlinks.unwrap_or(false),
            respect_ignore_files: self.respect_ignore_files.unwrap_or(true),
            git_ignore: self.git_ignore.unwrap_or(true),
            git_global: self.git_global.unwrap_or(true),
            git_exclude: self.git_exclude.unwrap_or(true),
            threads: self.threads,
            max_depth: self.max_depth,
            allowed_extensions: self
                .allowed_extensions
                .map(sanitize_extensions)
                .filter(|exts| !exts.is_empty()),
            context_label: self.context_label.clone(),
            global_ignores: self.global_ignores.unwrap_or_default(),
        };

        Ok((root, filesystem))
    }
}
