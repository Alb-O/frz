//! Core crate exports for building and running the `frz` terminal interface.
//!
//! The root module primarily re-exports types from the feature modules
//! so that embedders can configure the application without digging
//! through the module hierarchy.

pub mod app_dirs;
/// Feature modules containing vertical slices of functionality.
pub mod features;
/// Message envelopes and streaming primitives.
pub mod streams;

// Re-exports for public API
pub use features::filesystem_indexer::FilesystemOptions;
pub use features::tui_app::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig, run};

pub use crate::features::search_pipeline::{
	FileRow, SearchData, SearchOutcome, SearchSelection, TruncationStyle,
};
pub use crate::features::tui_app::components::{progress, rows as utils, tables, tabs};
pub use crate::features::tui_app::input::SearchInput;
pub use crate::features::tui_app::style::{StyleConfig, Theme, builtin_themes, default_theme};
