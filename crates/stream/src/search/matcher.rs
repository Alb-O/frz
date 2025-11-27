use std::cmp::{Ordering as CmpOrdering, Reverse};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use frizbee::{Config, match_list};

use super::channel::{MatchBatch, SearchStream};

/// Tunable thresholds shared across the search pipeline.
pub const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;

/// Maximum number of rows rendered in the result table.
pub const MAX_RENDERED_RESULTS: usize = 2_000;

/// Number of matches processed per scoring chunk.
pub const MATCH_CHUNK_SIZE: usize = 512;

/// Number of rows processed before emitting a heartbeat for empty queries.
pub const EMPTY_QUERY_BATCH: usize = 128;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum StreamPassResult {
	Completed,
	Aborted,
	HungUp,
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
pub struct ScoreAggregator<'a> {
	stream: SearchStream<'a>,
	heap: BinaryHeap<Reverse<RankedMatch>>,
	scratch: Vec<RankedMatch>,
	dirty: bool,
}

impl<'a> ScoreAggregator<'a> {
	/// Creates a new aggregator that will stream results through `stream`.
	pub fn new(stream: SearchStream<'a>) -> Self {
		Self {
			stream,
			heap: BinaryHeap::new(),
			scratch: Vec::new(),
			dirty: false,
		}
	}

	/// Inserts a scored match and marks the aggregator as dirty when the result set changes.
	pub fn push(&mut self, index: usize, score: u16) {
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
	pub fn flush_partial(&mut self) -> bool {
		if self.dirty {
			return self.emit(false);
		}
		true
	}

	/// Sends a final update for the query.
	pub fn finish(&mut self) -> bool {
		self.finish_with_completion(true)
	}

	/// Sends a final update for the query with a custom completion flag.
	pub fn finish_with_completion(&mut self, complete: bool) -> bool {
		self.emit(complete)
	}

	fn emit(&mut self, complete: bool) -> bool {
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

		self.dirty = false;
		self.stream.send_batch(
			MatchBatch {
				indices,
				ids: None,
				scores,
			},
			complete,
		)
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
pub struct AlphabeticalCollector<'a, F>
where
	F: FnMut(usize) -> String,
{
	stream: SearchStream<'a>,
	limit: usize,
	key_for_index: F,
	heap: BinaryHeap<AlphabeticalEntry>,
	scratch: Vec<AlphabeticalEntry>,
	dirty: bool,
}

impl<'a, F> AlphabeticalCollector<'a, F>
where
	F: FnMut(usize) -> String,
{
	/// Creates a collector that will emit at most [`MAX_RENDERED_RESULTS`] entries.
	pub fn new(stream: SearchStream<'a>, total: usize, key_for_index: F) -> Self {
		Self {
			stream,
			limit: MAX_RENDERED_RESULTS.min(total),
			key_for_index,
			heap: BinaryHeap::new(),
			scratch: Vec::new(),
			dirty: false,
		}
	}

	/// Inserts a candidate index when the collector still has capacity.
	pub fn insert(&mut self, index: usize) {
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
	pub fn flush_partial(&mut self) -> bool {
		if self.dirty {
			return self.emit(false);
		}
		true
	}

	/// Emits the final alphabetical set.
	pub fn finish(&mut self) -> bool {
		self.emit(true)
	}

	fn emit(&mut self, complete: bool) -> bool {
		if self.limit == 0 {
			self.dirty = false;
			return self.stream.send_batch(
				MatchBatch {
					indices: Vec::new(),
					ids: None,
					scores: Vec::new(),
				},
				complete,
			);
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

		self.dirty = false;
		self.stream.send_batch(
			MatchBatch {
				indices,
				ids: None,
				scores,
			},
			complete,
		)
	}
}

/// Represents a collection that can be searched via fuzzy matching.
pub trait Dataset {
	/// Total number of entries in the dataset.
	fn len(&self) -> usize;

	/// Returns true if the dataset contains no entries.
	fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Return the searchable key associated with `index`.
	fn key_for(&self, index: usize) -> &str;
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

/// Owned dataset that can be sent across threads for background refinement.
struct OwnedDataset {
	entries: Vec<String>,
}

impl OwnedDataset {
	fn new(entries: Vec<String>) -> Self {
		Self { entries }
	}
}

impl Dataset for OwnedDataset {
	fn len(&self) -> usize {
		self.entries.len()
	}

	fn key_for(&self, index: usize) -> &str {
		&self.entries[index]
	}
}

fn stream_matches_with_config<D>(
	dataset: D,
	trimmed: &str,
	config: &Config,
	aggregator: &mut ScoreAggregator<'_>,
	latest_query_id: &AtomicU64,
	stream_id: u64,
	mut owned_keys: Option<&mut Vec<String>>,
) -> StreamPassResult
where
	D: Dataset,
{
	let total = dataset.len();
	let mut haystacks = Vec::with_capacity(MATCH_CHUNK_SIZE);
	let mut offset = 0;
	while offset < total {
		if should_abort(stream_id, latest_query_id) {
			return StreamPassResult::Aborted;
		}

		let end = (offset + MATCH_CHUNK_SIZE).min(total);
		haystacks.clear();
		for index in offset..end {
			let key = dataset.key_for(index);
			haystacks.push(key);
			if let Some(keys) = owned_keys.as_deref_mut() {
				keys.push(key.to_owned());
			}
		}
		let matches = match_list(trimmed, &haystacks, config);
		for entry in matches {
			if entry.score == 0 {
				continue;
			}
			let index = offset + entry.index as usize;
			aggregator.push(index, entry.score);
		}

		if should_abort(stream_id, latest_query_id) {
			return StreamPassResult::Aborted;
		}
		if !aggregator.flush_partial() {
			return StreamPassResult::HungUp;
		}

		offset = end;
	}

	if should_abort(stream_id, latest_query_id) {
		return StreamPassResult::Aborted;
	}

	StreamPassResult::Completed
}

fn spawn_refined_search(
	query: String,
	haystacks: Vec<String>,
	stream: SearchStream<'_>,
	latest_query_id: Arc<AtomicU64>,
) {
	if haystacks.is_empty() {
		let _ = stream.send(Vec::new(), Vec::new(), true);
		return;
	}

	let tx = stream.clone_sender();
	let stream_id = stream.id();
	std::thread::spawn(move || {
		let stream = SearchStream::new(&tx, stream_id);
		let dataset = OwnedDataset::new(haystacks);

		let mut config = config_for_query(&query, dataset.len());
		config.prefilter = false;
		config.max_typos = None;

		let mut aggregator = ScoreAggregator::new(stream);
		let outcome = stream_matches_with_config(
			dataset,
			&query,
			&config,
			&mut aggregator,
			latest_query_id.as_ref(),
			stream_id,
			None,
		);

		if matches!(outcome, StreamPassResult::Completed)
			&& !should_abort(stream_id, latest_query_id.as_ref())
		{
			let _ = aggregator.finish();
		}
	});
}

/// Perform fuzzy matching on a dataset, emitting batches of ranked matches to the stream.
///
/// Returns `true` if streaming completed successfully, `false` if the receiver hung up.
pub fn stream_dataset<D, F>(
	dataset: D,
	query: &str,
	stream: SearchStream<'_>,
	latest_query_id: &Arc<AtomicU64>,
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
	if !config.prefilter {
		let mut aggregator = ScoreAggregator::new(stream);
		match stream_matches_with_config(
			dataset,
			trimmed,
			&config,
			&mut aggregator,
			latest_query_id.as_ref(),
			id,
			None,
		) {
			StreamPassResult::HungUp => return false,
			StreamPassResult::Aborted => return true,
			StreamPassResult::Completed => {}
		}

		return aggregator.finish();
	}

	let mut owned_keys = Vec::with_capacity(total);
	let mut aggregator = ScoreAggregator::new(stream.clone());
	match stream_matches_with_config(
		dataset,
		trimmed,
		&config,
		&mut aggregator,
		latest_query_id.as_ref(),
		id,
		Some(&mut owned_keys),
	) {
		StreamPassResult::HungUp => return false,
		StreamPassResult::Aborted => return true,
		StreamPassResult::Completed => {}
	}

	if !aggregator.finish_with_completion(false) {
		return false;
	}

	spawn_refined_search(
		trimmed.to_owned(),
		owned_keys,
		stream,
		Arc::clone(latest_query_id),
	);
	true
}

/// Stream results in alphabetical order when no query is provided.
///
/// Returns `true` if streaming completed successfully, `false` if the receiver hung up.
pub fn stream_alphabetical<F>(
	total: usize,
	stream: SearchStream<'_>,
	latest_query_id: &Arc<AtomicU64>,
	key_for_index: F,
) -> bool
where
	F: FnMut(usize) -> String,
{
	let id = stream.id();
	let mut collector = AlphabeticalCollector::new(stream, total, key_for_index);

	let mut processed = 0;
	for index in 0..total {
		if should_abort(id, latest_query_id.as_ref()) {
			return true;
		}
		collector.insert(index);
		processed += 1;
		if processed % EMPTY_QUERY_BATCH == 0 {
			if should_abort(id, latest_query_id.as_ref()) {
				return true;
			}
			if !collector.flush_partial() {
				return false;
			}
		}
	}

	if should_abort(id, latest_query_id.as_ref()) {
		return true;
	}

	collector.finish()
}

/// Check if this query has been superseded by a newer one.
pub fn should_abort(id: u64, latest_query_id: &AtomicU64) -> bool {
	latest_query_id.load(AtomicOrdering::Acquire) != id
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::search::SearchView;

	struct TestDataset(Vec<String>);

	#[derive(Default)]
	struct StubView {
		indices: Vec<usize>,
		scores: Vec<u16>,
		completions: Vec<bool>,
	}

	impl SearchView for StubView {
		fn replace_matches(&mut self, indices: Vec<usize>, scores: Vec<u16>) {
			self.indices = indices;
			self.scores = scores;
		}

		fn clear_matches(&mut self) {
			self.indices.clear();
			self.scores.clear();
		}

		fn record_completion(&mut self, complete: bool) {
			self.completions.push(complete);
		}
	}

	impl Dataset for TestDataset {
		fn len(&self) -> usize {
			self.0.len()
		}

		fn key_for(&self, index: usize) -> &str {
			&self.0[index]
		}
	}

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

	#[test]
	fn streams_empty_query_alphabetically() {
		use std::sync::Arc;
		use std::sync::mpsc::{Receiver, channel};

		let dataset = TestDataset(vec!["b".into(), "a".into()]);
		let (tx, rx): (_, Receiver<_>) = channel();
		let latest = Arc::new(AtomicU64::new(1));
		let stream = SearchStream::new(&tx, 1);
		stream_dataset(&dataset, "", stream, &latest, |idx| dataset.0[idx].clone());

		let envelope = rx.recv().unwrap();
		assert!(envelope.complete);
		let mut view = StubView {
			indices: Vec::new(),
			scores: Vec::new(),
			completions: Vec::new(),
		};
		envelope.dispatch(&mut view);

		assert_eq!(view.indices, vec![1, 0]); // alphabetical order
		assert_eq!(view.scores, vec![0, 0]);
		assert_eq!(view.completions, vec![true]);
	}

	#[test]
	fn refined_pass_signals_completion_after_prefilter() {
		use std::sync::mpsc::channel;
		use std::time::{Duration, Instant};

		let dataset = TestDataset(
			(0..=PREFILTER_ENABLE_THRESHOLD)
				.map(|i| format!("matching-file-{i}"))
				.collect(),
		);
		let (tx, rx) = channel();
		let latest = Arc::new(AtomicU64::new(1));
		let stream = SearchStream::new(&tx, 1);
		stream_dataset(&dataset, "matching", stream, &latest, |idx| {
			dataset.0[idx].clone()
		});

		let mut view = StubView::default();
		let start = Instant::now();
		while start.elapsed() < Duration::from_secs(2) {
			match rx.recv_timeout(Duration::from_millis(50)) {
				Ok(envelope) => {
					envelope.dispatch(&mut view);
					if view.completions.last() == Some(&true) {
						break;
					}
				}
				Err(_) => break,
			}
		}

		assert!(
			view.completions.first() == Some(&false),
			"prefiltered pass should emit a partial completion"
		);
		assert!(
			view.completions.iter().any(|complete| *complete),
			"refined pass should eventually mark the stream complete"
		);
	}
}
