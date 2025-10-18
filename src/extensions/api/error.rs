use thiserror::Error;

use super::search::SearchMode;

/// Errors that can occur when mutating the [`ExtensionCatalog`](crate::ExtensionCatalog).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExtensionCatalogError {
    /// An extension attempted to register an identifier that already exists in the catalog.
    #[error("extension id '{id}' is already registered")]
    DuplicateId { id: &'static str },

    /// An extension attempted to register a descriptor that is already present.
    #[error("extension for mode {mode:?} is already registered")]
    DuplicateMode { mode: SearchMode },

    /// A contribution attempted to register for a mode that already has an implementation.
    #[error("{contribution} contribution for mode {mode:?} is already registered")]
    ContributionConflict {
        contribution: &'static str,
        mode: SearchMode,
    },
}

impl ExtensionCatalogError {
    pub fn contribution_conflict(contribution: &'static str, mode: SearchMode) -> Self {
        Self::ContributionConflict { contribution, mode }
    }
}
