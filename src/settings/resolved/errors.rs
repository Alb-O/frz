use thiserror::Error;

use super::SettingSource;

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
