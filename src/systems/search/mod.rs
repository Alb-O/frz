mod aggregator;
mod alphabetical;
mod commands;
mod config;
mod streaming;
mod worker;

pub use commands::SearchStream;
pub(crate) use commands::{SearchCommand, SearchResult};
pub(crate) use config::config_for_query;
pub(crate) use streaming::{stream_facets, stream_files};
pub(crate) use worker::spawn;

pub(crate) const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;
pub(crate) const MAX_RENDERED_RESULTS: usize = 2_000;
const MATCH_CHUNK_SIZE: usize = 512;
const EMPTY_QUERY_BATCH: usize = 128;
