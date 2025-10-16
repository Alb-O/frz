use frz::{FilesystemOptions, SearchMode, UiConfig};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

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

#[derive(Debug, Error)]
#[error("invalid value for {key} from {origin}: {reason} (value: {value})")]
pub(super) struct ConfigError {
    key: &'static str,
    value: String,
    origin: SettingSource,
    reason: String,
}

impl ConfigError {
    fn invalid<K, V, R>(key: K, value: V, origin: SettingSource, reason: R) -> Self
    where
        K: Into<&'static str>,
        V: Into<String>,
        R: Into<String>,
    {
        Self {
            key: key.into(),
            value: value.into(),
            origin,
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum SettingSource {
    CliFlag(&'static str),
    Environment(&'static str),
    ConfigKey(&'static str),
}

impl fmt::Display for SettingSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CliFlag(flag) => write!(f, "CLI flag `{flag}`"),
            Self::Environment(var) => write!(f, "environment variable `{var}`"),
            Self::ConfigKey(key) => write!(f, "configuration key `{key}`"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct ConfigSources {
    pub(super) filesystem_threads: Option<SettingSource>,
    pub(super) filesystem_max_depth: Option<SettingSource>,
}

impl ConfigSources {
    fn source_for_threads(&self) -> SettingSource {
        self.filesystem_threads
            .clone()
            .unwrap_or(SettingSource::ConfigKey("filesystem.threads"))
    }

    fn source_for_max_depth(&self) -> SettingSource {
        self.filesystem_max_depth
            .clone()
            .unwrap_or(SettingSource::ConfigKey("filesystem.max_depth"))
    }
}

impl ResolvedConfig {
    pub(super) fn validate(&self, sources: &ConfigSources) -> Result<(), ConfigError> {
        if let Some(threads) = self.filesystem.threads
            && threads == 0
        {
            return Err(ConfigError::invalid(
                "filesystem.threads",
                threads.to_string(),
                sources.source_for_threads(),
                "must be greater than zero",
            ));
        }

        if let Some(max_depth) = self.filesystem.max_depth
            && max_depth == 0
        {
            return Err(ConfigError::invalid(
                "filesystem.max_depth",
                max_depth.to_string(),
                sources.source_for_max_depth(),
                "must be at least 1",
            ));
        }

        Ok(())
    }

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
                .map(|mode| mode.id().to_string())
                .unwrap_or_else(|| "(auto)".to_string())
        );
        if let Some(title) = &self.input_title {
            println!("  Prompt title: {title}");
        }
        if !self.initial_query.is_empty() {
            println!("  Initial query: {}", self.initial_query);
        }
        if let Some(headers) = &self.facet_headers {
            println!("  attribute headers: {}", headers.join(", "));
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

    #[test]
    fn validation_rejects_zero_threads() {
        let filesystem = FilesystemOptions {
            threads: Some(0),
            ..FilesystemOptions::default()
        };
        let config = ResolvedConfig {
            root: PathBuf::from("/tmp"),
            filesystem,
            input_title: None,
            initial_query: String::new(),
            theme: None,
            start_mode: None,
            ui: UiConfig::default(),
            facet_headers: None,
            file_headers: None,
        };

        let sources = ConfigSources {
            filesystem_threads: Some(SettingSource::CliFlag("--threads")),
            ..ConfigSources::default()
        };

        let err = config.validate(&sources).unwrap_err();
        assert!(matches!(err.key, "filesystem.threads"));
        let message = err.to_string();
        assert!(message.contains("value: 0"));
        assert!(message.contains("CLI flag"));
    }

    #[test]
    fn validation_rejects_zero_max_depth() {
        let filesystem = FilesystemOptions {
            max_depth: Some(0),
            ..FilesystemOptions::default()
        };
        let config = ResolvedConfig {
            root: PathBuf::from("/tmp"),
            filesystem,
            input_title: None,
            initial_query: String::new(),
            theme: None,
            start_mode: None,
            ui: UiConfig::default(),
            facet_headers: None,
            file_headers: None,
        };

        let sources = ConfigSources {
            filesystem_max_depth: Some(SettingSource::Environment("FRZ__FILESYSTEM__MAX_DEPTH")),
            ..ConfigSources::default()
        };

        let err = config.validate(&sources).unwrap_err();
        assert!(matches!(err.key, "filesystem.max_depth"));
        let message = err.to_string();
        assert!(message.contains("value: 0"));
        assert!(message.contains("environment variable"));
    }
}
