use std::time::Duration;

#[path = "../cache.rs"]
mod cache;
mod cached_stream;
mod options;
mod traversal;
mod update_batcher;

pub use options::FilesystemOptions;
pub use traversal::spawn_filesystem_index;

pub(super) const MIN_BATCH_SIZE: usize = 32;
pub(super) const MAX_BATCH_SIZE: usize = 1_024;
pub(super) const DISPATCH_INTERVAL: Duration = Duration::from_millis(120);
