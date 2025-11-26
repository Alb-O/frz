use std::collections::HashSet;
use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::Path;
use std::thread;

/// Configuration options for filesystem scanning and filtering.
#[derive(Debug, Clone)]
pub struct FilesystemOptions {
	/// Include hidden files and directories.
	pub include_hidden: bool,
	/// Follow symbolic links during traversal.
	pub follow_symlinks: bool,
	/// Respect .ignore files.
	pub respect_ignore_files: bool,
	/// Respect .gitignore files.
	pub git_ignore: bool,
	/// Respect global gitignore settings.
	pub git_global: bool,
	/// Respect git exclude files.
	pub git_exclude: bool,
	/// Directory names to always ignore.
	pub global_ignores: Vec<String>,
	/// Number of threads for parallel indexing.
	pub threads: Option<usize>,
	/// Maximum directory traversal depth.
	pub max_depth: Option<usize>,
	/// File extensions to filter by.
	pub allowed_extensions: Option<Vec<String>>,
	/// Label describing the search context.
	pub context_label: Option<String>,
}

impl Default for FilesystemOptions {
	fn default() -> Self {
		Self {
			include_hidden: true,
			follow_symlinks: false,
			respect_ignore_files: true,
			git_ignore: true,
			git_global: true,
			git_exclude: true,
			global_ignores: vec![
				".git".to_string(),
				"node_modules".to_string(),
				"target".to_string(),
				".venv".to_string(),
				".cache".to_string(),
				".local".to_string(),
				".cargo".to_string(),
				".mozilla".to_string(),
				".vscode-server".to_string(),
				".pki".to_string(),
				".dotnet".to_string(),
				".npm".to_string(),
				".rustup".to_string(),
				"__pycache__".to_string(),
				"sessionData".to_string(),
			],
			threads: None,
			max_depth: None,
			allowed_extensions: None,
			context_label: None,
		}
	}
}

impl FilesystemOptions {
	/// Set a default context label from the root path if not already configured.
	pub fn ensure_context_label(&mut self, root: &Path) -> Option<String> {
		if self.context_label.is_none() {
			self.context_label = Some(root.display().to_string());
		}
		self.context_label.clone()
	}

	/// Build a set of allowed extensions if configured.
	pub fn extension_filter(&self) -> Option<HashSet<String>> {
		self.allowed_extensions.as_ref().map(|extensions| {
			extensions
				.iter()
				.map(|ext| normalize_extension(ext))
				.filter(|ext| !ext.is_empty())
				.collect::<HashSet<_>>()
		})
	}

	/// Create a set of directory names to globally ignore.
	pub fn global_ignore_set(&self) -> HashSet<OsString> {
		self.global_ignores
			.iter()
			.map(|entry| OsString::from(entry.as_str()))
			.collect()
	}

	/// Resolve the effective thread count, defaulting to available parallelism.
	pub fn thread_count(&self) -> usize {
		self.threads
			.filter(|threads| *threads > 0)
			.unwrap_or_else(|| thread::available_parallelism().map_or(1, NonZeroUsize::get))
	}
}

/// Normalize an extension by trimming and removing leading dots.
pub fn normalize_extension(ext: &str) -> String {
	ext.trim().trim_start_matches('.').to_ascii_lowercase()
}
