pub mod app_dirs;
pub mod plugins;
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
pub use crate::tui::theme::{LIGHT, SLATE, SOLARIZED, Theme};
pub use systems::filesystem::FilesystemOptions;
pub use systems::search::plugin::{self as search_system, SearchStream};
pub use ui::run;
pub use ui::{PaneUiConfig, SearchUi, TabUiConfig, UiConfig};
