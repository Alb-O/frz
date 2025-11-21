use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Instant;

use super::super::{IndexKind, IndexResult, IndexStream, IndexUpdate};
use super::cache::CacheWriter;
use super::{DISPATCH_INTERVAL, MAX_BATCH_SIZE, MIN_BATCH_SIZE};
use crate::extensions::api::FileRow;

pub(super) struct UpdateBatcher {
	pending_files: Vec<FileRow>,
	indexed_files: usize,
	last_dispatch: Instant,
	emit_reset: bool,
	cache_writer: Option<CacheWriter>,
}

impl UpdateBatcher {
	pub fn new(emit_reset: bool, cache_writer: Option<CacheWriter>) -> Self {
		Self {
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

		self.indexed_files += 1;
		self.pending_files.push(file);
	}

	pub fn should_flush(&self) -> bool {
		if self.pending_files.len() >= batch_size_for(self.indexed_files) {
			return true;
		}

		if !self.emit_reset && self.pending_files.is_empty() {
			return false;
		}

		self.last_dispatch.elapsed() >= DISPATCH_INTERVAL
	}

	pub fn flush(&mut self, tx: &Sender<IndexResult>, complete: bool) -> bool {
		if !complete && !self.emit_reset && self.pending_files.is_empty() {
			return true;
		}

		let files_vec = std::mem::take(&mut self.pending_files);
		let files: Arc<[FileRow]> = files_vec.into();

		let progress = super::super::ProgressSnapshot {
			indexed_files: self.indexed_files,
			total_files: complete.then_some(self.indexed_files),
			complete,
		};

		let reset = self.emit_reset;
		if self.emit_reset {
			self.emit_reset = false;
		}

		let stream = IndexStream::new(tx, 0, IndexKind::Update);
		if !stream.send_update(
			IndexUpdate {
				files,
				progress,
				reset,
				cached_data: None,
			},
			complete,
		) {
			return false;
		}

		self.last_dispatch = Instant::now();
		true
	}

	pub fn finalize(self, tx: &Sender<IndexResult>) -> Option<CacheWriter> {
		let mut this = self;
		if !this.flush(tx, true) {
			return None;
		}
		this.cache_writer
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
