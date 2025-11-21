//! Core crate exports for building and running the `frz` terminal interface.
//!
//! The root module primarily re-exports types from the UI and extension
//! subsystems so that embedders can configure the application without digging
//! through the module hierarchy.

pub mod app_dirs;
pub mod extensions;
pub mod logging;
mod systems;
pub mod tui;
pub mod ui;

pub use systems::filesystem::FilesystemOptions;
pub use ui::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig, run};

pub use crate::extensions::api::{
	FileRow, SearchData, SearchOutcome, SearchSelection, TruncationStyle,
};
pub use crate::tui::components::{progress, tables, tabs};
pub use crate::tui::input::SearchInput;
pub use crate::tui::tables::rows as utils;
pub use crate::tui::theme::{Theme, builtin_themes, default_theme};
