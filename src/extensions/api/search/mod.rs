mod aggregator;
mod alphabetical;
mod data;
mod file;
mod fs;
mod mode;
mod outcome;
mod stream;
mod streaming;
mod util;

pub use data::SearchData;
pub use file::{AttributeRow, FileRow, TruncationStyle, tags_for_relative_path};
pub use fs::{Fs, FsIter, OsFs};
pub use mode::SearchMode;
pub use outcome::{ExtensionSelection, SearchOutcome, SearchSelection};
pub use stream::{MatchBatch, SearchResult, SearchStream, SearchView, SearchViewV2};
pub use streaming::{stream_attributes, stream_files};
pub use util::{
    EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD,
    config_for_query,
};
