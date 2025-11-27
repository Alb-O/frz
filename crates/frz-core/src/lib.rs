//! Core crate exports for building and running the `frz` terminal interface.
//!
//! The root module primarily re-exports types from the feature modules
//! so that embedders can configure the application without digging
//! through the module hierarchy.

pub mod app_dirs;
/// Feature modules containing vertical slices of functionality.
pub mod features;

// Re-exports for public API
pub use features::filesystem_indexer::FilesystemOptions;

pub use crate::features::search_pipeline::{
	FileRow, SearchData, SearchOutcome, SearchSelection, TruncationStyle,
};
