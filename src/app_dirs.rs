//! Resolve configuration, cache, and data directories for `frz`.
//!
//! The helpers in this module respect environment overrides while falling back
//! to platform-appropriate locations provided by the `dirs` crate.

use std::env;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

const APPLICATION: &str = "frz";

const CONFIG_DIR_ENV: &str = "FRZ_CONFIG_DIR";
const DATA_DIR_ENV: &str = "FRZ_DATA_DIR";
const CACHE_DIR_ENV: &str = "FRZ_CACHE_DIR";

/// Resolve an override directory from an environment variable.
///
/// An empty string is treated the same as an unset value so that callers can
/// use shell defaults without worrying about trailing whitespace.
fn dir_from_env(name: &str) -> Option<PathBuf> {
	let value = env::var_os(name)?;
	if value.is_empty() {
		None
	} else {
		Some(PathBuf::from(value))
	}
}

/// Return the configuration directory used to persist user preferences.
pub fn get_config_dir() -> Result<PathBuf> {
	if let Some(dir) = dir_from_env(CONFIG_DIR_ENV) {
		return Ok(dir);
	}

	let base = dirs::config_dir().ok_or_else(|| anyhow!("unable to determine config directory"))?;
	Ok(base.join(APPLICATION))
}

/// Return the data directory that stores search indexes and other assets.
pub fn get_data_dir() -> Result<PathBuf> {
	if let Some(dir) = dir_from_env(DATA_DIR_ENV) {
		return Ok(dir);
	}

	let base = dirs::data_dir().ok_or_else(|| anyhow!("unable to determine data directory"))?;
	Ok(base.join(APPLICATION))
}

/// Return the cache directory for temporary files and incremental state.
pub fn get_cache_dir() -> Result<PathBuf> {
	if let Some(dir) = dir_from_env(CACHE_DIR_ENV) {
		return Ok(dir);
	}

	let base = dirs::cache_dir().ok_or_else(|| anyhow!("unable to determine cache directory"))?;
	Ok(base.join(APPLICATION))
}
