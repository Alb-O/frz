use frz::{FilesystemOptions, SearchMode, UiConfig};
use std::path::PathBuf;

/// Application-ready configuration derived from user input, config files and
/// sensible defaults.
#[derive(Debug)]
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
    /// Print a human readable summary of the effective configuration.
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

fn bool_to_word(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frz::{FilesystemOptions, UiConfig};

    #[test]
    fn bool_to_word_matches_expectations() {
        assert_eq!(super::bool_to_word(true), "yes");
        assert_eq!(super::bool_to_word(false), "no");
    }

    #[test]
    fn summary_prints_without_panic() {
        let config = ResolvedConfig {
            root: PathBuf::from("/tmp"),
            filesystem: FilesystemOptions::default(),
            input_title: Some("Title".into()),
            initial_query: "foo".into(),
            theme: Some("dark".into()),
            start_mode: None,
            ui: UiConfig::default(),
            facet_headers: Some(vec!["col1".into()]),
            file_headers: Some(vec!["col2".into()]),
        };

        config.print_summary();
    }
}
