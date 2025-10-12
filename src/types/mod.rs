//! Types shared across the user interface and search pipelines.

mod facet;
mod file;
mod highlight;
mod search_data;
mod ui;

pub use facet::FacetRow;
pub use file::{FileRow, TruncationStyle};
pub(crate) use highlight::highlight_cell;
pub use search_data::{SearchData, SearchOutcome, SearchSelection};
pub use ui::{PaneUiConfig, SearchMode, UiConfig};

#[cfg(feature = "fs")]
pub(crate) use search_data::tags_for_relative_path;
