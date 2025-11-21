pub(crate) use crate::extensions::api::SearchResult;
use crate::extensions::api::{SearchData, SearchMode, StreamAction};

/// Commands understood by the background search worker.
#[derive(Debug)]
pub(crate) enum SearchCommand {
	/// Run a fuzzy search for the provided query and mode.
	Query {
		/// Identifier that allows the UI to correlate responses with the originating query.
		id: u64,
		/// User supplied query string.
		query: String,
		/// Which data set to search.
		mode: SearchMode,
	},
	/// Merge a fresh index update into the existing in-memory search data.
	Update(StreamAction<SearchData>),
	/// Stop the background worker thread.
	Shutdown,
}
