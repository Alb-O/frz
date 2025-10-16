#[cfg(feature = "fs")]
pub mod app_dirs;
pub mod input;
pub mod plugins;
mod systems;
pub use frz_tui::theme::*;
pub mod ui;

pub use frz_tui::tables::rows as utils;
pub use ui::components::progress;
pub use ui::components::tables;
pub use ui::components::tabs;

pub use frz_plugin_api::{
    FacetRow, FileRow, PluginSelection, SearchData, SearchMode, SearchOutcome, SearchPlugin,
    SearchPluginRegistry, SearchSelection, TruncationStyle,
};
pub use frz_tui::theme::{LIGHT, SLATE, SOLARIZED, Theme};
pub use input::SearchInput;
#[cfg(feature = "fs")]
pub use systems::filesystem::FilesystemOptions;
pub use systems::search::plugin::{self as search_system, SearchStream};
pub use ui::run;
pub use ui::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig};
