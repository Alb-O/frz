use std::env;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use directories::ProjectDirs;

const QUALIFIER: &str = "io";
const ORGANIZATION: &str = "albo";
const APPLICATION: &str = "frz";

const CONFIG_DIR_ENV: &str = "FRZ_CONFIG_DIR";
const DATA_DIR_ENV: &str = "FRZ_DATA_DIR";

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| anyhow!("unable to determine project directories for frz"))
}

fn dir_from_env(name: &str) -> Option<PathBuf> {
    let value = env::var_os(name)?;
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    if let Some(dir) = dir_from_env(CONFIG_DIR_ENV) {
        return Ok(dir);
    }

    Ok(project_dirs()?.config_local_dir().to_path_buf())
}

pub fn get_data_dir() -> Result<PathBuf> {
    if let Some(dir) = dir_from_env(DATA_DIR_ENV) {
        return Ok(dir);
    }

    Ok(project_dirs()?.data_local_dir().to_path_buf())
}
