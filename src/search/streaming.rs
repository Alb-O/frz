use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use frizbee::match_list;

use super::aggregator::ScoreAggregator;
use super::alphabetical::AlphabeticalCollector;
use super::config::config_for_query;
use super::file::FileRow;
use super::stream::SearchStream;
use super::{EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, SearchData};

/// Represents a collection that can be searched via fuzzy matching.
pub trait Dataset {
	/// Total number of entries in the dataset.
	fn len(&self) -> usize;

	/// Return the searchable key associated with `index`.
	fn key_for(&self, index: usize) -> &str;
}

impl Dataset for [FileRow] {
	fn len(&self) -> usize {
		<[FileRow]>::len(self)
	}

	fn key_for(&self, index: usize) -> &str {
		self[index].search_text()
	}
}

impl<T> Dataset for &T
where
	T: Dataset + ?Sized,
{
	fn len(&self) -> usize {
		<T as Dataset>::len(*self)
	}

	fn key_for(&self, index: usize) -> &str {
		<T as Dataset>::key_for(*self, index)
	}
}

/// Streams file matches for the given query back to the UI thread.
pub fn stream_files(
	data: &SearchData,
	query: &str,
	stream: SearchStream<'_>,
	latest_query_id: &AtomicU64,
) -> bool {
	let files = data.files.as_slice();
	stream_dataset(files, query, stream, latest_query_id, move |index| {
		files[index].path.clone()
	})
}

fn stream_dataset<D, F>(
	dataset: D,
	query: &str,
	stream: SearchStream<'_>,
	latest_query_id: &AtomicU64,
	alphabetical_key: F,
) -> bool
where
	D: Dataset,
	F: FnMut(usize) -> String,
{
	let id = stream.id();
	let trimmed = query.trim();
	if trimmed.is_empty() {
		return stream_alphabetical(dataset.len(), stream, latest_query_id, alphabetical_key);
	}

	let total = dataset.len();
	let config = config_for_query(trimmed, total);
	let mut aggregator = ScoreAggregator::new(stream);
	let mut haystacks = Vec::with_capacity(MATCH_CHUNK_SIZE);
	let mut offset = 0;
	while offset < total {
		if should_abort(id, latest_query_id) {
			return true;
		}

		let end = (offset + MATCH_CHUNK_SIZE).min(total);
		haystacks.clear();
		for index in offset..end {
			haystacks.push(dataset.key_for(index));
		}
		let matches = match_list(trimmed, &haystacks, &config);
		for entry in matches {
			if entry.score == 0 {
				continue;
			}
			let index = offset + entry.index as usize;
			aggregator.push(index, entry.score);
		}

		if should_abort(id, latest_query_id) {
			return true;
		}
		if !aggregator.flush_partial() {
			return false;
		}

		offset = end;
	}

	if should_abort(id, latest_query_id) {
		return true;
	}

	aggregator.finish()
}

fn stream_alphabetical<F>(
	total: usize,
	stream: SearchStream<'_>,
	latest_query_id: &AtomicU64,
	key_for_index: F,
) -> bool
where
	F: FnMut(usize) -> String,
{
	let id = stream.id();
	let mut collector = AlphabeticalCollector::new(stream, total, key_for_index);

	let mut processed = 0;
	for index in 0..total {
		if should_abort(id, latest_query_id) {
			return true;
		}
		collector.insert(index);
		processed += 1;
		if processed % EMPTY_QUERY_BATCH == 0 {
			if should_abort(id, latest_query_id) {
				return true;
			}
			if !collector.flush_partial() {
				return false;
			}
		}
	}

	if should_abort(id, latest_query_id) {
		return true;
	}

	collector.finish()
}

fn should_abort(id: u64, latest_query_id: &AtomicU64) -> bool {
	latest_query_id.load(AtomicOrdering::Acquire) != id
}
