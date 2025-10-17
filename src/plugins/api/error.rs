use thiserror::Error;

use super::search::SearchMode;

/// Errors that can occur when mutating the [`SearchPluginRegistry`](crate::SearchPluginRegistry).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginRegistryError {
    /// A plugin attempted to register an identifier that already exists in the registry.
    #[error("plugin id '{id}' is already registered")]
    DuplicateId { id: &'static str },

    /// A plugin attempted to register a descriptor that is already present.
    #[error("plugin for mode {mode:?} is already registered")]
    DuplicateMode { mode: SearchMode },

    /// A capability attempted to register for a mode that already has an implementation.
    #[error("{capability} capability for mode {mode:?} is already registered")]
    CapabilityConflict {
        capability: &'static str,
        mode: SearchMode,
    },
}

impl PluginRegistryError {
    pub fn capability_conflict(capability: &'static str, mode: SearchMode) -> Self {
        Self::CapabilityConflict { capability, mode }
    }
}
