#[cfg(feature = "fs")]
pub mod app_dirs;
mod indexing;
pub mod input;
mod search;
pub mod theme;
pub mod types;
pub mod ui;

pub use ui::components::progress;
pub use ui::components::tables;
pub use ui::components::tables::rows as utils;
pub use ui::components::tabs;

#[cfg(feature = "fs")]
pub use indexing::FilesystemOptions;
pub use input::SearchInput;
pub use theme::{LIGHT, SLATE, SOLARIZED, Theme};
pub use types::{
    FacetRow, FileRow, PaneUiConfig, SearchData, SearchMode, SearchOutcome, SearchSelection,
    UiConfig,
};
pub use ui::SearchUi;
pub use ui::run;
