use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use ignore::{DirEntry, Error as IgnoreError, WalkBuilder, WalkState};

use crate::plugins::api::{AttributeRow, FileRow, SearchData, tags_for_relative_path};

use super::super::{IndexUpdate, ProgressSnapshot};
use super::FilesystemOptions;
use super::cache::{CacheHandle, CacheWriter};
use super::cached_stream::stream_cached_entry;
use super::update_batcher::UpdateBatcher;

pub fn spawn_filesystem_index(
    root: PathBuf,
    mut options: FilesystemOptions,
) -> Result<(SearchData, Receiver<IndexUpdate>)> {
    let (tx, rx) = mpsc::channel();

    let cache_handle = CacheHandle::resolve(&root, &options);
    let mut data = SearchData::new();

    let context_label = options.ensure_context_label(&root);
    data.context_label = context_label.clone();

    let should_reset = cache_handle.is_some();
    let cache_handle_for_thread = cache_handle.clone();

    thread::spawn(move || {
        let mut reindex_delay = Duration::ZERO;
        let mut preview_complete = false;
        let mut preview_file_count = None;

        if let Some(handle) = cache_handle_for_thread.as_ref() {
            if let Some(mut preview) = handle.load_preview() {
                reindex_delay = preview.reindex_delay();
                let preview_is_complete = preview.is_complete();
                preview_file_count = Some(preview.data.files.len());

                if preview.data.context_label.is_none() {
                    preview.data.context_label = context_label.clone();
                }

                let files: Arc<[FileRow]> = preview.data.files.clone().into();
                let attributes: Arc<[AttributeRow]> = preview.data.attributes.clone().into();
                let progress = ProgressSnapshot {
                    indexed_attributes: attributes.len(),
                    indexed_files: files.len(),
                    total_attributes: preview_is_complete.then_some(attributes.len()),
                    total_files: preview_is_complete.then_some(files.len()),
                    complete: preview_is_complete,
                };

                if !files.is_empty() || !attributes.is_empty() {
                    let _ = tx.send(IndexUpdate {
                        files,
                        attributes,
                        progress,
                        reset: true,
                        cached_data: Some(preview.data),
                    });
                }

                preview_complete = preview_is_complete;
            }

            if !preview_complete && let Some(mut entry) = handle.load() {
                reindex_delay = entry.reindex_delay();

                if entry.data.context_label.is_none() {
                    entry.data.context_label = context_label.clone();
                }

                stream_cached_entry(entry, preview_file_count, &tx);
            }
        }

        if !reindex_delay.is_zero() {
            thread::sleep(reindex_delay);
        }

        let (file_tx, file_rx) = mpsc::channel::<FileRow>();
        let walker_root = Arc::new(root);
        let threads = options.thread_count();
        let extension_filter = options.extension_filter().map(Arc::new);
        let update_tx = tx;

        let cache_writer = cache_handle_for_thread
            .as_ref()
            .and_then(|handle| handle.writer(context_label.clone()));
        let aggregator = thread::spawn(move || {
            let mut batcher = UpdateBatcher::new(should_reset, cache_writer);

            while let Ok(file) = file_rx.recv() {
                batcher.record_file(file);

                if batcher.should_flush() && batcher.flush(&update_tx, false).is_err() {
                    return None::<CacheWriter>;
                }
            }

            batcher.finalize(&update_tx).unwrap_or_default()
        });

        let global_ignores = Arc::new(options.global_ignore_set());

        WalkBuilder::new(walker_root.as_path())
            .hidden(!options.include_hidden)
            .follow_links(options.follow_symlinks)
            .git_ignore(options.git_ignore)
            .git_global(options.git_global)
            .git_exclude(options.git_exclude)
            .ignore(options.respect_ignore_files)
            .parents(true)
            .threads(threads)
            .max_depth(options.max_depth)
            .build_parallel()
            .run(|| {
                let sender = file_tx.clone();
                let root = Arc::clone(&walker_root);
                let extension_filter = extension_filter.clone();
                let global_ignores = Arc::clone(&global_ignores);
                Box::new(move |entry: Result<DirEntry, IgnoreError>| {
                    if let Ok(entry) = entry {
                        let Some(file_type) = entry.file_type() else {
                            return WalkState::Continue;
                        };
                        if !file_type.is_file() {
                            return WalkState::Continue;
                        }

                        let path = entry.path();
                        if path
                            .components()
                            .any(|comp| global_ignores.contains(comp.as_os_str()))
                        {
                            return WalkState::Continue;
                        }
                        let relative = path.strip_prefix(root.as_path()).unwrap_or(path);
                        if let Some(filter) = extension_filter.as_ref() {
                            let extension = relative
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext.to_ascii_lowercase());
                            if extension.as_ref().is_none_or(|ext| !filter.contains(ext)) {
                                return WalkState::Continue;
                            }
                        }
                        let tags = tags_for_relative_path(relative);
                        let relative_display = relative.to_string_lossy().replace('\\', "/");
                        let file = FileRow::filesystem(relative_display, tags);
                        if sender.send(file).is_err() {
                            return WalkState::Quit;
                        }
                    }

                    WalkState::Continue
                })
            });

        drop(file_tx);
        if let Ok(Some(writer)) = aggregator.join() {
            let _ = writer.finish();
        }
    });

    Ok((data, rx))
}
