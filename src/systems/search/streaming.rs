use frizbee::match_list;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use super::aggregator::ScoreAggregator;
use super::alphabetical::AlphabeticalCollector;
use super::commands::SearchStream;
use super::{EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, config::config_for_query};
use crate::types::SearchData;

/// Streams facet matches for the given query back to the UI thread.
pub fn stream_facets(
    data: &SearchData,
    query: &str,
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let id = stream.id();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return stream_alphabetical_facets(data, stream, latest_query_id);
    }

    let config = config_for_query(trimmed, data.facets.len());
    let mut aggregator = ScoreAggregator::new(stream);
    let mut haystacks = Vec::with_capacity(MATCH_CHUNK_SIZE);
    let mut offset = 0;
    for chunk in data.facets.chunks(MATCH_CHUNK_SIZE) {
        if should_abort(id, latest_query_id) {
            return true;
        }
        haystacks.clear();
        haystacks.extend(chunk.iter().map(|facet| facet.name.as_str()));
        let matches = match_list(trimmed, &haystacks, config);
        for entry in matches {
            if entry.score == 0 {
                continue;
            }
            let index = offset + entry.index_in_haystack as usize;
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

/// Streams file matches for the given query back to the UI thread.
pub fn stream_files(
    data: &SearchData,
    query: &str,
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let id = stream.id();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return stream_alphabetical_files(data, stream, latest_query_id);
    }

    let config = config_for_query(trimmed, data.files.len());
    let mut aggregator = ScoreAggregator::new(stream);
    let mut haystacks = Vec::with_capacity(MATCH_CHUNK_SIZE);
    let mut offset = 0;
    for chunk in data.files.chunks(MATCH_CHUNK_SIZE) {
        if should_abort(id, latest_query_id) {
            return true;
        }
        haystacks.clear();
        haystacks.extend(chunk.iter().map(|file| file.search_text()));
        let matches = match_list(trimmed, &haystacks, config);
        for entry in matches {
            if entry.score == 0 {
                continue;
            }
            let index = offset + entry.index_in_haystack as usize;
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
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let id = stream.id();
    let mut collector = AlphabeticalCollector::new(stream, data.facets.len(), |index| {
        data.facets[index].name.clone()
    });

    let mut processed = 0;
    for index in 0..data.facets.len() {
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
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let id = stream.id();
    let mut collector = AlphabeticalCollector::new(stream, data.files.len(), |index| {
        data.files[index].path.clone()
    });

    let mut processed = 0;
    for index in 0..data.files.len() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FacetRow, FileRow};

    fn sample_data() -> SearchData {
        SearchData::default()
            .with_facets(vec![FacetRow::new("alpha", 1), FacetRow::new("beta", 1)])
            .with_files(vec![
                FileRow::new("src/lib.rs", Vec::<String>::new()),
                FileRow::new("src/main.rs", Vec::<String>::new()),
            ])
    }

    #[test]
    fn aborts_when_query_id_changes() {
        let data = sample_data();
        let (tx, rx) = std::sync::mpsc::channel();
        let latest = AtomicU64::new(42);

        let stream = SearchStream::new(&tx, 41, crate::plugins::builtin::files::mode());
        assert!(stream_files(&data, "alpha", stream, &latest));
        assert!(rx.try_recv().is_err());
    }
}
