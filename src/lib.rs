//! Core crate exports for building and running the `frz` terminal interface.
//!
//! The root module primarily re-exports types from the UI and plugin
//! subsystems so that embedders can configure the application without digging
//! through the module hierarchy.

pub mod app_dirs;
pub mod plugins;
pub mod previewers;
mod systems;
pub mod tui;
pub mod ui;

pub use crate::plugins::api::{
    AttributeRow, FileRow, PluginSelection, SearchData, SearchMode, SearchOutcome, SearchPlugin,
    SearchPluginRegistry, SearchSelection, TruncationStyle,
};
pub use crate::tui::components::{progress, tables, tabs};
pub use crate::tui::input::SearchInput;
pub use crate::tui::tables::rows as utils;
pub use crate::tui::theme::{Theme, builtin_themes, default_theme};
pub use systems::filesystem::FilesystemOptions;
pub use systems::search::plugin::{self as search_system, SearchStream};
pub use ui::run;
pub use ui::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig};
