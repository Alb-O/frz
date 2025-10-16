use std::collections::HashSet;
use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::Path;
use std::thread;

#[derive(Debug, Clone)]
pub struct FilesystemOptions {
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub respect_ignore_files: bool,
    pub git_ignore: bool,
    pub git_global: bool,
    pub git_exclude: bool,
    pub global_ignores: Vec<String>,
    pub threads: Option<usize>,
    pub max_depth: Option<usize>,
    pub allowed_extensions: Option<Vec<String>>,
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
    pub fn ensure_context_label(&mut self, root: &Path) -> Option<String> {
        if self.context_label.is_none() {
            self.context_label = Some(root.display().to_string());
        }
        self.context_label.clone()
    }

    pub fn extension_filter(&self) -> Option<HashSet<String>> {
        self.allowed_extensions.as_ref().map(|extensions| {
            extensions
                .iter()
                .map(|ext| normalize_extension(ext))
                .filter(|ext| !ext.is_empty())
                .collect::<HashSet<_>>()
        })
    }

    pub fn global_ignore_set(&self) -> HashSet<OsString> {
        self.global_ignores
            .iter()
            .map(|entry| OsString::from(entry.as_str()))
            .collect()
    }

    pub fn thread_count(&self) -> usize {
        self.threads
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| thread::available_parallelism().map_or(1, NonZeroUsize::get))
    }
}

pub fn normalize_extension(ext: &str) -> String {
    ext.trim().trim_start_matches('.').to_ascii_lowercase()
}
