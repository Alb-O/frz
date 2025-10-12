pub mod app;
#[cfg(feature = "fs")]
pub mod app_dirs;
mod indexing;
pub mod input;
pub mod progress;
mod search;
pub mod searcher;
pub mod tables;
pub mod tabs;
pub mod theme;
pub mod types;
pub mod utils;

pub use app::run;
#[cfg(feature = "fs")]
pub use indexing::FilesystemOptions;
pub use input::SearchInput;
pub use searcher::Searcher;
pub use theme::{LIGHT, SLATE, SOLARIZED, Theme};
pub use types::{
    FacetRow, FileRow, PaneUiConfig, SearchData, SearchMode, SearchOutcome, SearchSelection,
    UiConfig,
};
