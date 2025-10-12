use anyhow::{Result, anyhow};

use crate::cli::CliArgs;

use super::raw::RawConfig;
use super::resolved::ResolvedConfig;
use super::sources::build_config;

/// Load configuration by combining CLI arguments, config files and environment
/// variables.
pub fn load(cli: &CliArgs) -> Result<ResolvedConfig> {
    let builder = build_config(cli)?;
    let mut raw: RawConfig = builder
        .try_deserialize()
        .map_err(|err| anyhow!("failed to deserialize configuration: {err}"))?;
    raw.apply_cli_overrides(cli);
    raw.resolve()
}
