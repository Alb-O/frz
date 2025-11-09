//! Configuration loading and resolution utilities.
//!
//! This module provides the configuration pipeline. `load` is the primary entry point and
//! returns a [`ResolvedConfig`] that is used by the application.

use anyhow::{Result, anyhow};
use crate::cli::CliArgs;

mod raw;
mod resolved;
mod sources;
mod ui;
mod util;

pub use resolved::ResolvedConfig;

/// Load configuration by combining CLI arguments, config files and environment
/// variables.
pub fn load(cli: &CliArgs) -> Result<ResolvedConfig> {
    let builder = sources::build_config(cli)?;
    let mut raw: raw::RawConfig = builder
        .try_deserialize()
        .map_err(|err| anyhow!("failed to deserialize configuration: {err}"))?;
    raw.apply_cli_overrides(cli);
    raw.resolve(cli)
}
