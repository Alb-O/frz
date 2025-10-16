#[cfg(feature = "fs")]
pub mod app_dirs;
pub mod input;
pub mod plugins;
mod systems;
pub mod theme;
pub mod types;
pub mod ui;

pub use ui::components::progress;
pub use ui::components::tables;
pub use ui::components::tables::rows as utils;
pub use ui::components::tabs;

pub use input::SearchInput;
pub use plugins::{SearchMode, SearchPlugin, SearchPluginRegistry};
#[cfg(feature = "fs")]
pub use systems::filesystem::FilesystemOptions;
pub use systems::search::plugin::{self as search_system, SearchStream};
pub use theme::{LIGHT, SLATE, SOLARIZED, Theme};
pub use types::{
    FacetRow, FileRow, PaneUiConfig, PluginSelection, SearchData, SearchOutcome, SearchSelection,
    TabUiConfig, UiConfig,
};
pub use ui::SearchUi;
pub use ui::run;
