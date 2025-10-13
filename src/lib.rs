#[cfg(feature = "fs")]
pub mod app_dirs;
mod indexing;
pub mod input;
pub mod progress;
mod search;
pub mod tables;
pub mod tabs;
pub mod theme;
pub mod types;
pub mod ui;
pub mod utils;

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
