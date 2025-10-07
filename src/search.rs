use std::cmp::{Ordering as CmpOrdering, Reverse};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use frizbee::{Config, match_list};

use crate::indexing::{IndexUpdate, merge_update};
use crate::types::{SearchData, SearchMode};

pub(crate) const PREFILTER_ENABLE_THRESHOLD: usize = 1_000;
pub(crate) const MAX_RENDERED_RESULTS: usize = 2_000;
const MATCH_CHUNK_SIZE: usize = 512;
const EMPTY_QUERY_BATCH: usize = 128;

#[derive(Debug)]
pub(crate) enum SearchCommand {
    Query {
        id: u64,
        query: String,
        mode: SearchMode,
    },
    Update(IndexUpdate),
    Shutdown,
}

#[derive(Debug)]
pub(crate) struct SearchResult {
    pub(crate) id: u64,
    pub(crate) mode: SearchMode,
    pub(crate) indices: Vec<usize>,
    pub(crate) scores: Vec<u16>,
    #[allow(dead_code)]
    pub(crate) complete: bool,
}

pub(crate) fn spawn(
    mut data: SearchData,
) -> (
    Sender<SearchCommand>,
    Receiver<SearchResult>,
    Arc<AtomicU64>,
) {
    let (command_tx, command_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let latest_query_id = Arc::new(AtomicU64::new(0));
    let thread_latest = Arc::clone(&latest_query_id);

    thread::spawn(move || {
        while let Ok(command) = command_rx.recv() {
            match command {
                SearchCommand::Query { id, query, mode } => {
                    if !process_query(&data, &query, mode, id, &result_tx, &thread_latest) {
                        break;
                    }
                }
                SearchCommand::Update(update) => {
                    merge_update(&mut data, &update);
                }
                SearchCommand::Shutdown => break,
            }
        }
    });

    (command_tx, result_rx, latest_query_id)
}

fn process_query(
    data: &SearchData,
    query: &str,
    mode: SearchMode,
    id: u64,
    tx: &Sender<SearchResult>,
    latest_query_id: &AtomicU64,
) -> bool {
    match mode {
        SearchMode::Facets => stream_facets(data, query, id, tx, latest_query_id),
        SearchMode::Files => stream_files(data, query, id, tx, latest_query_id),
    }
}

fn stream_facets(
    data: &SearchData,
    query: &str,
    id: u64,
    tx: &Sender<SearchResult>,
    latest_query_id: &AtomicU64,
) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return stream_alphabetical_facets(data, id, tx, latest_query_id);
    }

    let config = config_for_query(trimmed, data.facets.len());
    let mut aggregator = ScoreAggregator::new(id, SearchMode::Facets, tx);
    let mut offset = 0;
    for chunk in data.facets.chunks(MATCH_CHUNK_SIZE) {
        if should_abort(id, latest_query_id) {
            return true;
        }
        let haystacks: Vec<&str> = chunk.iter().map(|facet| facet.name.as_str()).collect();
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
        offset += chunk.len();
    }

    if should_abort(id, latest_query_id) {
        return true;
    }

    aggregator.finish()
}

fn stream_files(
    data: &SearchData,
    query: &str,
    id: u64,
    tx: &Sender<SearchResult>,
    latest_query_id: &AtomicU64,
) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return stream_alphabetical_files(data, id, tx, latest_query_id);
    }

    let config = config_for_query(trimmed, data.files.len());
    let mut aggregator = ScoreAggregator::new(id, SearchMode::Files, tx);
    let mut offset = 0;
    for chunk in data.files.chunks(MATCH_CHUNK_SIZE) {
        if should_abort(id, latest_query_id) {
            return true;
        }
        let haystacks: Vec<&str> = chunk.iter().map(|file| file.search_text()).collect();
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
        offset += chunk.len();
    }

    if should_abort(id, latest_query_id) {
        return true;
    }

    aggregator.finish()
}

fn stream_alphabetical_facets(
    data: &SearchData,
    id: u64,
    tx: &Sender<SearchResult>,
    latest_query_id: &AtomicU64,
) -> bool {
    let mut collector =
        AlphabeticalCollector::new(id, SearchMode::Facets, tx, data.facets.len(), |index| {
            data.facets[index].name.clone()
        });

    let mut processed = 0;
    for (index, _) in data.facets.iter().enumerate() {
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

fn stream_alphabetical_files(
    data: &SearchData,
    id: u64,
    tx: &Sender<SearchResult>,
    latest_query_id: &AtomicU64,
) -> bool {
    let mut collector =
        AlphabeticalCollector::new(id, SearchMode::Files, tx, data.files.len(), |index| {
            data.files[index].path.clone()
        });

    let mut processed = 0;
    for (index, _) in data.files.iter().enumerate() {
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

struct ScoreAggregator<'a> {
    id: u64,
    mode: SearchMode,
    tx: &'a Sender<SearchResult>,
    heap: BinaryHeap<Reverse<RankedMatch>>,
    scratch: Vec<RankedMatch>,
    dirty: bool,
    sent_any: bool,
}

impl<'a> ScoreAggregator<'a> {
    fn new(id: u64, mode: SearchMode, tx: &'a Sender<SearchResult>) -> Self {
        Self {
            id,
            mode,
            tx,
            heap: BinaryHeap::new(),
            scratch: Vec::new(),
            dirty: false,
            sent_any: false,
        }
    }

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

    fn flush_partial(&mut self) -> bool {
        if !self.dirty {
            return true;
        }
        self.emit(false)
    }

    fn finish(&mut self) -> bool {
        if self.dirty || !self.sent_any {
            if !self.emit(true) {
                return false;
            }
        } else if self.heap.is_empty() {
            if !self.emit(true) {
                return false;
            }
        } else if !self.emit(true) {
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

        match self.tx.send(SearchResult {
            id: self.id,
            mode: self.mode,
            indices,
            scores,
            complete,
        }) {
            Ok(()) => {
                self.sent_any = true;
                self.dirty = false;
                true
            }
            Err(_) => false,
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

struct AlphabeticalCollector<'a, F>
where
    F: Fn(usize) -> String,
{
    id: u64,
    mode: SearchMode,
    tx: &'a Sender<SearchResult>,
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
    fn new(
        id: u64,
        mode: SearchMode,
        tx: &'a Sender<SearchResult>,
        total: usize,
        key_for_index: F,
    ) -> Self {
        Self {
            id,
            mode,
            tx,
            limit: MAX_RENDERED_RESULTS.min(total),
            key_for_index,
            heap: BinaryHeap::new(),
            scratch: Vec::new(),
            dirty: false,
            sent_any: false,
        }
    }

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
        } else if let Some(mut current_max) = self.heap.peek_mut() {
            if entry < *current_max {
                *current_max = entry;
                self.dirty = true;
            }
        }
    }

    fn flush_partial(&mut self) -> bool {
        if !self.dirty {
            return true;
        }
        self.emit(false)
    }

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
            return self
                .tx
                .send(SearchResult {
                    id: self.id,
                    mode: self.mode,
                    indices: Vec::new(),
                    scores: Vec::new(),
                    complete,
                })
                .is_ok();
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

        match self.tx.send(SearchResult {
            id: self.id,
            mode: self.mode,
            indices,
            scores,
            complete,
        }) {
            Ok(()) => {
                self.sent_any = true;
                self.dirty = false;
                true
            }
            Err(_) => false,
        }
    }
}

pub(crate) fn config_for_query(query: &str, dataset_len: usize) -> Config {
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
