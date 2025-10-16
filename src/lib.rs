pub mod app_dirs;
pub mod plugins;
mod systems;
pub mod ui;

pub use frz_tui::components::{progress, tables, tabs};
pub use frz_tui::input::SearchInput;
pub use frz_tui::tables::rows as utils;

pub use frz_plugin_api::{
    AttributeRow, FileRow, PluginSelection, SearchData, SearchMode, SearchOutcome, SearchPlugin,
    SearchPluginRegistry, SearchSelection, TruncationStyle,
};
pub use frz_tui::theme::{LIGHT, SLATE, SOLARIZED, Theme};
pub use systems::filesystem::FilesystemOptions;
pub use systems::search::plugin::{self as search_system, SearchStream};
pub use ui::run;
pub use ui::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig};
