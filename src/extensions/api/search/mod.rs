mod aggregator;
mod alphabetical;
mod config;
mod data;
mod file;
mod fs;
mod identity;
mod outcome;
mod stream;
mod streaming;
mod tuning;

pub use config::config_for_query;
pub use data::{FILES_DATASET_KEY, SearchData};
pub use file::{FileRow, TruncationStyle};
pub use fs::{Fs, FsIter, OsFs};
pub use outcome::{SearchOutcome, SearchSelection};
pub use stream::{MatchBatch, SearchMarker, SearchResult, SearchStream, SearchView, SearchViewV2};
pub use streaming::stream_files;
pub use tuning::{
	EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD,
};
