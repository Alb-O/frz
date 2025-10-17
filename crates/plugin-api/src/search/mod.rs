mod aggregator;
mod alphabetical;
mod attribute;
mod config;
mod data;
mod file;
mod mode;
mod outcome;
mod stream;
mod streaming;

pub use attribute::AttributeRow;
pub use config::config_for_query;
pub use data::SearchData;
pub use file::{FileRow, TruncationStyle, tags_for_relative_path};
pub use mode::SearchMode;
pub use outcome::{PluginSelection, SearchOutcome, SearchSelection};
pub use stream::{SearchResult, SearchStream};
pub use streaming::{stream_attributes, stream_files};

pub const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;
pub const MAX_RENDERED_RESULTS: usize = 2_000;
const MATCH_CHUNK_SIZE: usize = 512;
const EMPTY_QUERY_BATCH: usize = 128;
