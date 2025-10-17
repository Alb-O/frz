use frizbee::match_list;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use super::{
    EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, SearchData, aggregator::ScoreAggregator,
    alphabetical::AlphabeticalCollector, config::config_for_query, stream::SearchStream,
};

/// Streams attribute matches for the given query back to the UI thread.
pub fn stream_attributes(
    data: &SearchData,
    query: &str,
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let id = stream.id();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return stream_alphabetical_attributes(data, stream, latest_query_id);
    }

    let config = config_for_query(trimmed, data.attributes.len());
    let mut aggregator = ScoreAggregator::new(stream);
    let mut haystacks = Vec::with_capacity(MATCH_CHUNK_SIZE);
    let mut offset = 0;
    for chunk in data.attributes.chunks(MATCH_CHUNK_SIZE) {
        if should_abort(id, latest_query_id) {
            return true;
        }
        haystacks.clear();
        haystacks.extend(chunk.iter().map(|attribute| attribute.name.as_str()));
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

fn stream_alphabetical_attributes(
    data: &SearchData,
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let id = stream.id();
    let mut collector = AlphabeticalCollector::new(stream, data.attributes.len(), |index| {
        data.attributes[index].name.clone()
    });

    let mut processed = 0;
    for index in 0..data.attributes.len() {
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
    use crate::plugins::api::{
        TableContext, TableDescriptor,
        descriptors::{SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition},
        search::{AttributeRow, FileRow, SearchMode},
    };
    use std::sync::mpsc::channel;

    struct NullDataset;

    impl SearchPluginDataset for NullDataset {
        fn key(&self) -> &'static str {
            "stream"
        }

        fn total_count(&self, _data: &SearchData) -> usize {
            0
        }

        fn build_table<'a>(&self, _context: TableContext<'a>) -> TableDescriptor<'a> {
            TableDescriptor::new(Vec::new(), Vec::new(), Vec::new())
        }
    }

    static DATASET: NullDataset = NullDataset;

    static DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
        id: "stream",
        ui: SearchPluginUiDefinition {
            tab_label: "Stream",
            mode_title: "",
            hint: "",
            table_title: "",
            count_label: "",
        },
        dataset: &DATASET,
    };

    fn mode() -> SearchMode {
        SearchMode::from_descriptor(&DESCRIPTOR)
    }

    fn sample_data() -> SearchData {
        SearchData::default()
            .with_attributes(vec![
                AttributeRow::new("alpha", 1),
                AttributeRow::new("beta", 1),
            ])
            .with_files(vec![
                FileRow::new("src/lib.rs", Vec::<String>::new()),
                FileRow::new("src/main.rs", Vec::<String>::new()),
            ])
    }

    #[test]
    fn aborts_when_query_id_changes() {
        let data = sample_data();
        let (tx, rx) = channel();
        let latest = AtomicU64::new(42);

        let stream = SearchStream::new(&tx, 41, mode());
        assert!(stream_files(&data, "alpha", stream, &latest));
        assert!(rx.try_recv().is_err());
    }
}
