use super::MAX_RENDERED_RESULTS;
use super::commands::SearchStream;
use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;

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
    F: Fn(usize) -> String,
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
    F: Fn(usize) -> String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SearchMode;

    #[test]
    fn keeps_smallest_entries() {
        let (tx, rx) = std::sync::mpsc::channel();
        let stream = SearchStream::new(&tx, 9, SearchMode::FILES);
        let mut collector =
            AlphabeticalCollector::new(stream, 5, |idx| ["z", "b", "a", "y", "c"][idx].to_string());

        for index in 0..5 {
            collector.insert(index);
        }
        assert!(collector.finish());

        let result = rx.try_recv().expect("collector should emit");
        assert_eq!(result.indices, vec![2, 1, 4, 3, 0]);
        assert_eq!(result.scores, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn handles_empty_dataset() {
        let (tx, rx) = std::sync::mpsc::channel();
        let stream = SearchStream::new(&tx, 3, SearchMode::FACETS);
        let mut collector = AlphabeticalCollector::new(stream, 0, |_| "".into());
        assert!(collector.finish());
        let result = rx.try_recv().expect("empty collector should emit");
        assert!(result.indices.is_empty());
        assert!(result.scores.is_empty());
    }
}
