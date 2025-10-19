use frz::{FilesystemOptions, SearchMode, UiConfig};
use std::path::PathBuf;

mod errors;
mod sources;
mod summary;
mod validation;

pub(crate) use errors::ConfigError;
pub(crate) use sources::{ConfigSources, SettingSource};

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
