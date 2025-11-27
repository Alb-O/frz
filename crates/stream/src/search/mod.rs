//! Non-blocking fuzzy search streamer built on top of the base streaming
//! primitives.

mod channel;
mod matcher;

pub use channel::{
	MatchBatch, SearchAction, SearchMarker, SearchResult, SearchStream, SearchView, SearchViewV2,
};
pub use matcher::{
	AlphabeticalCollector, Dataset, EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, MAX_RENDERED_RESULTS,
	PREFILTER_ENABLE_THRESHOLD, ScoreAggregator, config_for_query, stream_alphabetical,
	stream_dataset,
};
