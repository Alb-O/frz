use frz::{FilesystemOptions, SearchMode, UiConfig};
use std::path::PathBuf;
use std::fmt;
use thiserror::Error;

mod summary;
mod validation;

#[derive(Debug, Error)]
#[error("invalid value for {key} from {origin}: {reason} (value: {value})")]
pub(crate) struct ConfigError {
    pub(crate) key: &'static str,
    pub(crate) value: String,
    pub(crate) origin: SettingSource,
    pub(crate) reason: String,
}

impl ConfigError {
    pub(crate) fn invalid<K, V, R>(key: K, value: V, origin: SettingSource, reason: R) -> Self
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
pub(crate) enum SettingSource {
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
pub(crate) struct ConfigSources {
    pub(crate) filesystem_threads: Option<SettingSource>,
    pub(crate) filesystem_max_depth: Option<SettingSource>,
}

impl ConfigSources {
    pub(crate) fn source_for_threads(&self) -> SettingSource {
        self.filesystem_threads
            .clone()
            .unwrap_or(SettingSource::ConfigKey("filesystem.threads"))
    }

    pub(crate) fn source_for_max_depth(&self) -> SettingSource {
        self.filesystem_max_depth
            .clone()
            .unwrap_or(SettingSource::ConfigKey("filesystem.max_depth"))
    }
}

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
    pub git_modifications: bool,
}

impl ResolvedConfig {
    pub(super) fn validate(&self, sources: &ConfigSources) -> Result<(), ConfigError> {
        validation::validate(self, sources)
    }

    /// Print a human readable summary of the effective configuration.
    pub fn print_summary(&self) {
        summary::print_summary(self);
    }
}
