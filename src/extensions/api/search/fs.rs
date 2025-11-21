use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::{io, thread};

use ignore::{DirEntry, Error as IgnoreError, WalkBuilder, WalkState};

/// Iterator over filesystem paths produced by an [`Fs`] implementation.
pub struct FsIter {
	rx: mpsc::Receiver<io::Result<PathBuf>>,
	worker: Option<thread::JoinHandle<()>>,
}

impl Iterator for FsIter {
	type Item = io::Result<PathBuf>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.rx.recv() {
			Ok(item) => Some(item),
			Err(_) => {
				// Ensure the worker thread has finished before terminating the iterator.
				if let Some(handle) = self.worker.take() {
					let _ = handle.join();
				}
				None
			}
		}
	}
}

impl Drop for FsIter {
	fn drop(&mut self) {
		if let Some(handle) = self.worker.take() {
			let _ = handle.join();
		}
	}
}

/// Abstraction over filesystem traversal used by search data builders.
///
/// Implementations can fabricate directory trees for tests or forward to the OS
/// with additional behaviour such as `.gitignore` support.
pub trait Fs {
	type Iter: Iterator<Item = io::Result<PathBuf>> + Send + 'static;

	fn walk(&self, root: &Path) -> io::Result<Self::Iter>;
}

/// OS-backed implementation that honours `.gitignore` defaults via `ignore`.
pub struct OsFs;

impl Fs for OsFs {
	type Iter = FsIter;

	fn walk(&self, root: &Path) -> io::Result<Self::Iter> {
		let root = root.to_path_buf();
		let walker_root = Arc::new(root.clone());
		let (tx, rx) = mpsc::channel();
		let threads = thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);

		let worker = thread::spawn(move || {
			WalkBuilder::new(walker_root.as_path())
				.hidden(false)
				.git_ignore(true)
				.git_global(true)
				.git_exclude(true)
				.ignore(true)
				.parents(true)
				.threads(threads)
				.build_parallel()
				.run(|| {
					let sender = tx.clone();
					let root = Arc::clone(&walker_root);
					Box::new(move |entry: Result<DirEntry, IgnoreError>| {
						match entry {
							Ok(entry) => {
								let Some(file_type) = entry.file_type() else {
									return WalkState::Continue;
								};
								if !file_type.is_file() {
									return WalkState::Continue;
								}

								let path = entry.path();
								let relative = path.strip_prefix(root.as_path()).unwrap_or(path);
								let relative = relative.to_path_buf();
								if sender.send(Ok(relative)).is_err() {
									return WalkState::Quit;
								}
							}
							Err(err) => {
								let io_err = io::Error::other(err.to_string());
								if sender.send(Err(io_err)).is_err() {
									return WalkState::Quit;
								}
							}
						}

						WalkState::Continue
					})
				});
		});

		Ok(FsIter {
			rx,
			worker: Some(worker),
		})
	}
}
