//! Fuzzy matching engine and result aggregation.

use std::cmp::{Ordering as CmpOrdering, Reverse};
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use frizbee::{Config, match_list};

use super::channel::SearchStream;
use super::file::FileRow;
use super::{
	EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, MAX_RENDERED_RESULTS, PREFILTER_ENABLE_THRESHOLD,
	SearchData,
};

/// Builds fuzzy matching options for the provided query and dataset size.
pub fn config_for_query(query: &str, dataset_len: usize) -> Config {
	let mut config = Config {
		prefilter: false,
		..Config::default()
	};

	let length = query.chars().count();
	let mut allowed_typos: u16 = match length {
		0 => 0,
		1 => 0,
		2..=4 => 1,
		5..=7 => 2,
		8..=12 => 3,
		_ => 4,
	};
	if let Ok(max_reasonable) = u16::try_from(length.saturating_sub(1)) {
		allowed_typos = allowed_typos.min(max_reasonable);
	}

	if dataset_len >= PREFILTER_ENABLE_THRESHOLD {
		config.prefilter = true;
		config.max_typos = Some(allowed_typos);
	} else {
		config.prefilter = false;
		config.max_typos = None;
	}

	config.sort = false;

	config
}

#[derive(Clone, Eq, PartialEq)]
struct RankedMatch {
	index: usize,
	score: u16,
}

impl Ord for RankedMatch {
	fn cmp(&self, other: &Self) -> CmpOrdering {
		self.score
			.cmp(&other.score)
			.then_with(|| other.index.cmp(&self.index))
	}
}

impl PartialOrd for RankedMatch {
	fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
		Some(self.cmp(other))
	}
}

/// Maintains the highest scoring matches for a particular query.
struct ScoreAggregator<'a> {
	stream: SearchStream<'a>,
	heap: BinaryHeap<Reverse<RankedMatch>>,
	scratch: Vec<RankedMatch>,
	dirty: bool,
	sent_any: bool,
}

impl<'a> ScoreAggregator<'a> {
	/// Creates a new aggregator that will stream results through `stream`.
	fn new(stream: SearchStream<'a>) -> Self {
		Self {
			stream,
			heap: BinaryHeap::new(),
			scratch: Vec::new(),
			dirty: false,
			sent_any: false,
		}
	}

	/// Inserts a scored match and marks the aggregator as dirty when the result set changes.
	fn push(&mut self, index: usize, score: u16) {
		if self.insert(RankedMatch { index, score }) {
			self.dirty = true;
		}
	}

	fn insert(&mut self, entry: RankedMatch) -> bool {
		if self.heap.len() < MAX_RENDERED_RESULTS {
			self.heap.push(Reverse(entry));
			true
		} else if let Some(mut current_min) = self.heap.peek_mut() {
			if entry > current_min.0 {
				*current_min = Reverse(entry);
				true
			} else {
				false
			}
		} else {
			false
		}
	}

	/// Emits an incremental update when new matches were observed.
	fn flush_partial(&mut self) -> bool {
		if !self.dirty {
			return true;
		}
		self.emit(false)
	}

	/// Sends a final update for the query.
	fn finish(&mut self) -> bool {
		if !self.emit(true) {
			return false;
		}
		true
	}

	fn emit(&mut self, complete: bool) -> bool {
		if self.heap.is_empty() && !complete && self.sent_any {
			self.dirty = false;
			return true;
		}

		self.scratch.clear();
		self.scratch
			.extend(self.heap.iter().map(|entry| entry.0.clone()));
		self.scratch
			.sort_unstable_by(|a, b| b.score.cmp(&a.score).then_with(|| a.index.cmp(&b.index)));

		let mut indices = Vec::with_capacity(self.scratch.len());
		let mut scores = Vec::with_capacity(self.scratch.len());
		for entry in &self.scratch {
			indices.push(entry.index);
			scores.push(entry.score);
		}

		if self.stream.send(indices, scores, complete) {
			self.sent_any = true;
			self.dirty = false;
			true
		} else {
			false
		}
	}
}

#[derive(Clone, Eq, PartialEq)]
struct AlphabeticalEntry {
	index: usize,
	key: String,
}

impl Ord for AlphabeticalEntry {
	fn cmp(&self, other: &Self) -> CmpOrdering {
		self.key
			.cmp(&other.key)
			.then_with(|| self.index.cmp(&other.index))
	}
}

impl PartialOrd for AlphabeticalEntry {
	fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
		Some(self.cmp(other))
	}
}

/// Collects the lexicographically smallest entries for an empty query.
struct AlphabeticalCollector<'a, F>
where
	F: FnMut(usize) -> String,
{
	stream: SearchStream<'a>,
	limit: usize,
	key_for_index: F,
	heap: BinaryHeap<AlphabeticalEntry>,
	scratch: Vec<AlphabeticalEntry>,
	dirty: bool,
	sent_any: bool,
}

impl<'a, F> AlphabeticalCollector<'a, F>
where
	F: FnMut(usize) -> String,
{
	/// Creates a collector that will emit at most [`MAX_RENDERED_RESULTS`] entries.
	fn new(stream: SearchStream<'a>, total: usize, key_for_index: F) -> Self {
		Self {
			stream,
			limit: MAX_RENDERED_RESULTS.min(total),
			key_for_index,
			heap: BinaryHeap::new(),
			scratch: Vec::new(),
			dirty: false,
			sent_any: false,
		}
	}

	/// Inserts a candidate index when the collector still has capacity.
	fn insert(&mut self, index: usize) {
		if self.limit == 0 {
			return;
		}
		let entry = AlphabeticalEntry {
			index,
			key: (self.key_for_index)(index),
		};
		if self.heap.len() < self.limit {
			self.heap.push(entry);
			self.dirty = true;
		} else if let Some(mut current_max) = self.heap.peek_mut()
			&& entry < *current_max
		{
			*current_max = entry;
			self.dirty = true;
		}
	}

	/// Emits an incremental update when new items were inserted.
	fn flush_partial(&mut self) -> bool {
		if !self.dirty {
			return true;
		}
		self.emit(false)
	}

	/// Emits the final alphabetical set.
	fn finish(&mut self) -> bool {
		if self.limit == 0 {
			return self.emit(true);
		}

		if !self.emit(true) {
			return false;
		}
		true
	}

	fn emit(&mut self, complete: bool) -> bool {
		if self.limit == 0 {
			return self.stream.send(Vec::new(), Vec::new(), complete);
		}

		self.scratch.clear();
		self.scratch.extend(self.heap.iter().cloned());
		self.scratch
			.sort_unstable_by(|a, b| a.key.cmp(&b.key).then_with(|| a.index.cmp(&b.index)));

		let mut indices = Vec::with_capacity(self.scratch.len());
		for entry in &self.scratch {
			indices.push(entry.index);
		}
		let scores = vec![0; indices.len()];

		if self.stream.send(indices, scores, complete) {
			self.sent_any = true;
			self.dirty = false;
			true
		} else {
			false
		}
	}
}

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

/// Perform fuzzy matching on a dataset, emitting batches of ranked matches to the stream.
///
/// Returns `true` if streaming completed successfully, `false` if the receiver hung up.
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

/// Stream results in alphabetical order when no query is provided.
///
/// Returns `true` if streaming completed successfully, `false` if the receiver hung up.
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

/// Check if this query has been superseded by a newer one.
fn should_abort(id: u64, latest_query_id: &AtomicU64) -> bool {
	latest_query_id.load(AtomicOrdering::Acquire) != id
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn enables_prefilter_for_large_datasets() {
		let config = config_for_query("example", PREFILTER_ENABLE_THRESHOLD);
		assert!(config.prefilter);
		assert_eq!(config.max_typos, Some(2));
	}

	#[test]
	fn disables_prefilter_for_small_datasets() {
		let config = config_for_query("example", PREFILTER_ENABLE_THRESHOLD - 1);
		assert!(!config.prefilter);
		assert_eq!(config.max_typos, None);
	}
}
