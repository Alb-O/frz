use std::cmp::{Ordering as CmpOrdering, Reverse};
use std::collections::BinaryHeap;

use super::MAX_RENDERED_RESULTS;
use super::stream::SearchStream;

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
pub(super) struct ScoreAggregator<'a> {
	stream: SearchStream<'a>,
	heap: BinaryHeap<Reverse<RankedMatch>>,
	scratch: Vec<RankedMatch>,
	dirty: bool,
	sent_any: bool,
}

impl<'a> ScoreAggregator<'a> {
	/// Creates a new aggregator that will stream results through `stream`.
	pub(super) fn new(stream: SearchStream<'a>) -> Self {
		Self {
			stream,
			heap: BinaryHeap::new(),
			scratch: Vec::new(),
			dirty: false,
			sent_any: false,
		}
	}

	/// Inserts a scored match and marks the aggregator as dirty when the result set changes.
	pub(super) fn push(&mut self, index: usize, score: u16) {
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
	pub(super) fn flush_partial(&mut self) -> bool {
		if !self.dirty {
			return true;
		}
		self.emit(false)
	}

	/// Sends a final update for the query.
	pub(super) fn finish(&mut self) -> bool {
		if !self.emit(true) {
			return false;
		}
		true
	}

	pub(super) fn emit(&mut self, complete: bool) -> bool {
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
