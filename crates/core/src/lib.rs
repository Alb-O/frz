//! Core crate exports for building and running the `frz` terminal interface.
//!
//! The root module primarily re-exports types from the feature modules
//! so that embedders can configure the application without digging
//! through the module hierarchy.

pub mod app_dirs;
pub mod filesystem_indexer;
pub mod search_pipeline;

pub use crate::search_pipeline::{
	FileRow, SearchData, SearchOutcome, SearchSelection, TruncationStyle,
};
