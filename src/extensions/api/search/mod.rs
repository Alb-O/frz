mod aggregator;
mod alphabetical;
mod config;
mod data;
mod file;
mod fs;
mod identity;
mod mode;
mod outcome;
mod stream;
mod streaming;
mod tuning;

pub use config::config_for_query;
pub use data::SearchData;
pub use file::{FileRow, TruncationStyle, tags_for_relative_path};
pub use fs::{Fs, FsIter, OsFs};
pub use mode::SearchMode;
pub use outcome::{SearchOutcome, SearchSelection};
pub use stream::{MatchBatch, SearchResult, SearchStream, SearchView, SearchViewV2};
pub use streaming::stream_files;
pub use tuning::{
	EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD,
};
