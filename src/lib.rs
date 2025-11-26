//! Core crate exports for building and running the `frz` terminal interface.
//!
//! The root module primarily re-exports types from the UI and extension
//! subsystems so that embedders can configure the application without digging
//! through the module hierarchy.

pub mod app_dirs;
pub mod search;
pub mod streams;
mod systems;
pub mod ui;

pub use systems::filesystem::FilesystemOptions;
pub use ui::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig, run};

pub use crate::search::{FileRow, SearchData, SearchOutcome, SearchSelection, TruncationStyle};
pub use crate::ui::components::{progress, rows as utils, tables, tabs};
pub use crate::ui::input::SearchInput;
pub use crate::ui::style::{StyleConfig, Theme, builtin_themes, default_theme};
