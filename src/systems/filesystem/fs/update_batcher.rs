use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::time::Instant;

use crate::types::{FacetRow, FileRow};

use super::super::IndexUpdate;
use super::cache::CacheWriter;
use super::{DISPATCH_INTERVAL, MAX_BATCH_SIZE, MIN_BATCH_SIZE};

pub(super) struct UpdateBatcher {
    facet_counts: BTreeMap<String, usize>,
    pending_facets: BTreeMap<String, usize>,
    pending_files: Vec<FileRow>,
    indexed_files: usize,
    last_dispatch: Instant,
    emit_reset: bool,
    cache_writer: Option<CacheWriter>,
}

impl UpdateBatcher {
    pub fn new(emit_reset: bool, cache_writer: Option<CacheWriter>) -> Self {
        Self {
            facet_counts: BTreeMap::new(),
            pending_facets: BTreeMap::new(),
            pending_files: Vec::new(),
            indexed_files: 0,
            last_dispatch: Instant::now(),
            emit_reset,
            cache_writer,
        }
    }

    pub fn record_file(&mut self, file: FileRow) {
        if let Some(writer) = &mut self.cache_writer {
            writer.record(&file);
        }

        for tag in &file.tags {
            let count = self.facet_counts.entry(tag.clone()).or_insert(0);
            *count += 1;
            self.pending_facets.insert(tag.clone(), *count);
        }

        self.indexed_files += 1;
        self.pending_files.push(file);
    }

    pub fn should_flush(&self) -> bool {
        if self.pending_files.len() >= batch_size_for(self.indexed_files) {
            return true;
        }

        if !self.emit_reset && self.pending_files.is_empty() && self.pending_facets.is_empty() {
            return false;
        }

        self.last_dispatch.elapsed() >= DISPATCH_INTERVAL
    }

    pub fn flush(
        &mut self,
        tx: &Sender<IndexUpdate>,
        complete: bool,
    ) -> Result<(), Box<mpsc::SendError<IndexUpdate>>> {
        if !complete
            && !self.emit_reset
            && self.pending_files.is_empty()
            && self.pending_facets.is_empty()
        {
            return Ok(());
        }

        let files_vec = std::mem::take(&mut self.pending_files);
        let files: Arc<[FileRow]> = files_vec.into();
        let facets: Arc<[FacetRow]> = if self.pending_facets.is_empty() {
            Arc::default()
        } else {
            let collected: Vec<FacetRow> = self
                .pending_facets
                .iter()
                .map(|(name, count)| FacetRow::new(name.clone(), *count))
                .collect();
            self.pending_facets.clear();
            collected.into()
        };

        let progress = super::super::ProgressSnapshot {
            indexed_facets: self.facet_counts.len(),
            indexed_files: self.indexed_files,
            total_facets: complete.then_some(self.facet_counts.len()),
            total_files: complete.then_some(self.indexed_files),
            complete,
        };

        let reset = self.emit_reset;
        if self.emit_reset {
            self.emit_reset = false;
        }

        tx.send(IndexUpdate {
            files,
            facets,
            progress,
            reset,
            cached_data: None,
        })
        .map_err(Box::new)?;

        self.last_dispatch = Instant::now();
        Ok(())
    }

    pub fn finalize(
        self,
        tx: &Sender<IndexUpdate>,
    ) -> Result<Option<CacheWriter>, Box<mpsc::SendError<IndexUpdate>>> {
        let mut this = self;
        this.flush(tx, true)?;
        Ok(this.cache_writer)
    }
}

fn batch_size_for(indexed_files: usize) -> usize {
    if indexed_files < 1_024 {
        MIN_BATCH_SIZE
    } else if indexed_files < 16_384 {
        256
    } else {
        MAX_BATCH_SIZE
    }
}
