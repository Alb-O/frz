use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;

use super::MAX_RENDERED_RESULTS;
use super::stream::SearchStream;

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
pub(super) struct AlphabeticalCollector<'a, F>
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
	pub(super) fn new(stream: SearchStream<'a>, total: usize, key_for_index: F) -> Self {
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
	pub(super) fn insert(&mut self, index: usize) {
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
	pub(super) fn flush_partial(&mut self) -> bool {
		if !self.dirty {
			return true;
		}
		self.emit(false)
	}

	/// Emits the final alphabetical set.
	pub(super) fn finish(&mut self) -> bool {
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
