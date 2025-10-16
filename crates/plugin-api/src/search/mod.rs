mod aggregator;
mod alphabetical;
mod config;
mod stream;
mod streaming;

pub use config::config_for_query;
pub use stream::{SearchResult, SearchStream};
pub use streaming::{stream_facets, stream_files};

pub const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;
pub const MAX_RENDERED_RESULTS: usize = 2_000;
const MATCH_CHUNK_SIZE: usize = 512;
const EMPTY_QUERY_BATCH: usize = 128;
