use thiserror::Error;

use crate::types::SearchMode;

/// Errors that can occur when mutating the [`SearchPluginRegistry`](crate::SearchPluginRegistry).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginRegistryError {
    /// A plugin attempted to register an identifier that already exists in the registry.
    #[error("plugin id '{id}' is already registered")]
    DuplicateId { id: &'static str },

    /// A plugin attempted to register a descriptor that is already present.
    #[error("plugin for mode {mode:?} is already registered")]
    DuplicateMode { mode: SearchMode },

    /// A plugin attempted to register a preview split for a mode that already has one.
    #[error("preview split for mode {mode:?} is already registered")]
    DuplicatePreviewSplit { mode: SearchMode },
}
