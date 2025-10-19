use frizbee::match_list;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use super::{
    EMPTY_QUERY_BATCH, MATCH_CHUNK_SIZE, SearchData, aggregator::ScoreAggregator,
    alphabetical::AlphabeticalCollector, attribute::AttributeRow, config::config_for_query,
    file::FileRow, stream::SearchStream,
};

/// Represents a collection that can be searched via fuzzy matching.
pub trait Dataset {
    /// Total number of entries in the dataset.
    fn len(&self) -> usize;

    /// Return the searchable key associated with `index`.
    fn key_for(&self, index: usize) -> &str;
}

impl Dataset for [AttributeRow] {
    fn len(&self) -> usize {
        <[AttributeRow]>::len(self)
    }

    fn key_for(&self, index: usize) -> &str {
        &self[index].name
    }
}

impl Dataset for [FileRow] {
    fn len(&self) -> usize {
        <[FileRow]>::len(self)
    }

    fn key_for(&self, index: usize) -> &str {
        self[index].search_text()
    }
}

impl<'a, T> Dataset for &'a T
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

/// Streams attribute matches for the given query back to the UI thread.
pub fn stream_attributes(
    data: &SearchData,
    query: &str,
    stream: SearchStream<'_>,
    latest_query_id: &AtomicU64,
) -> bool {
    let attributes = data.attributes.as_slice();
    stream_dataset(attributes, query, stream, latest_query_id, move |index| {
        attributes[index].name.clone()
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::api::{
        TableContext, TableDescriptor,
        descriptors::{ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition},
        search::{AttributeRow, FileRow, SearchMode},
    };
    use std::sync::mpsc::channel;

    struct NullDataset;

    impl ExtensionDataset for NullDataset {
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

    static DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
        id: "stream",
        ui: ExtensionUiDefinition {
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
