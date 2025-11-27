//! Background worker for generating syntax-highlighted previews.
//!
//! This module provides an asynchronous preview generation system that runs in a
//! background thread, preventing the UI from blocking while bat processes files.
//!
//! The worker maintains an LRU cache of recently previewed files, allowing instant
//! display when revisiting files without re-reading from disk or re-highlighting.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;

use bat::assets::HighlightingAssets;

use super::content::PreviewContent;
use super::highlight::highlight_with_bat;
#[cfg(feature = "media-preview")]
use super::image::{ImagePreview, is_image_file};

/// Maximum number of previews to keep in the LRU cache.
const CACHE_CAPACITY: usize = 32;

/// Commands sent to the preview worker thread.
pub enum PreviewCommand {
	/// Request a preview for a file.
	Generate {
		/// Unique ID for this preview request (for deduplication).
		id: u64,
		/// Path to the file to preview.
		path: PathBuf,
		/// Optional bat theme name.
		theme: Option<String>,
		/// Maximum number of lines to render.
		max_lines: usize,
	},
	/// Shut down the worker thread.
	Shutdown,
}

/// Cache key combining path and theme for proper cache invalidation.
#[derive(Clone, Hash, Eq, PartialEq)]
struct CacheKey {
	path: PathBuf,
	theme: Option<String>,
}

/// Simple LRU cache for preview content.
struct PreviewCache {
	/// Map from cache key to (order, content).
	entries: HashMap<CacheKey, (u64, PreviewContent)>,
	/// Counter for LRU ordering (higher = more recent).
	order: u64,
	/// Maximum number of entries.
	capacity: usize,
}

impl PreviewCache {
	fn new(capacity: usize) -> Self {
		Self {
			entries: HashMap::with_capacity(capacity),
			order: 0,
			capacity,
		}
	}

	/// Get a cached preview if available, updating its LRU order.
	fn get(&mut self, key: &CacheKey) -> Option<PreviewContent> {
		if let Some((order, content)) = self.entries.get_mut(key) {
			self.order += 1;
			*order = self.order;
			Some(content.clone())
		} else {
			None
		}
	}

	/// Insert a preview into the cache, evicting the oldest if at capacity.
	fn insert(&mut self, key: CacheKey, content: PreviewContent) {
		// Evict oldest entry if at capacity
		if self.entries.len() >= self.capacity
			&& !self.entries.contains_key(&key)
			&& let Some(oldest_key) = self
				.entries
				.iter()
				.min_by_key(|(_, (order, _))| *order)
				.map(|(k, _)| k.clone())
		{
			self.entries.remove(&oldest_key);
		}

		self.order += 1;
		self.entries.insert(key, (self.order, content));
	}
}
/// Results sent back from the preview worker thread.
pub struct PreviewResult {
	/// The ID of the request this result corresponds to.
	pub id: u64,
	/// The generated preview content.
	pub content: PreviewContent,
}

/// Spawns the background preview worker thread and returns communication channels.
pub fn spawn() -> (Sender<PreviewCommand>, Receiver<PreviewResult>) {
	let (command_tx, command_rx) = std::sync::mpsc::channel();
	let (result_tx, result_rx) = std::sync::mpsc::channel();

	thread::Builder::new()
		.name("preview-worker".into())
		.spawn(move || worker_loop(command_rx, result_tx))
		.expect("failed to spawn preview worker thread");

	(command_tx, result_rx)
}

fn worker_loop(command_rx: Receiver<PreviewCommand>, result_tx: Sender<PreviewResult>) {
	// Load highlighting assets once and reuse them for all previews.
	// This is the most expensive part of bat initialization.
	let assets = HighlightingAssets::from_binary();

	// LRU cache for recently previewed files
	let mut cache = PreviewCache::new(CACHE_CAPACITY);

	while let Ok(command) = command_rx.recv() {
		match command {
			PreviewCommand::Generate {
				id,
				path,
				theme,
				max_lines,
			} => {
				// Before doing any work, drain the channel to get the latest request.
				// This avoids processing stale requests when the user navigates quickly.
				let (final_id, final_path, final_theme, final_max_lines) =
					drain_to_latest(&command_rx, id, path, theme, max_lines);

				let cache_key = CacheKey {
					path: final_path.clone(),
					theme: final_theme.clone(),
				};

				// Check cache first for instant response
				let content = if let Some(cached) = cache.get(&cache_key) {
					cached
				} else {
					let generated = generate_preview_impl(
						&final_path,
						final_theme.as_deref(),
						final_max_lines,
						&assets,
					);
					cache.insert(cache_key, generated.clone());
					generated
				};

				// If the receiver is gone, just exit gracefully
				if result_tx
					.send(PreviewResult {
						id: final_id,
						content,
					})
					.is_err()
				{
					break;
				}
			}
			PreviewCommand::Shutdown => break,
		}
	}
}

/// Drain the command channel and return the most recent Generate request.
///
/// This allows us to skip stale requests when the user navigates quickly,
/// avoiding expensive processing of files the user has already moved past.
fn drain_to_latest(
	rx: &Receiver<PreviewCommand>,
	mut id: u64,
	mut path: PathBuf,
	mut theme: Option<String>,
	mut max_lines: usize,
) -> (u64, PathBuf, Option<String>, usize) {
	// Non-blocking drain of any pending requests
	loop {
		match rx.try_recv() {
			Ok(PreviewCommand::Generate {
				id: new_id,
				path: new_path,
				theme: new_theme,
				max_lines: new_max_lines,
			}) => {
				// Found a newer request, use it instead
				id = new_id;
				path = new_path;
				theme = new_theme;
				max_lines = new_max_lines;
			}
			Ok(PreviewCommand::Shutdown) => {
				// Put shutdown back for the main loop to handle
				// (We can't easily do this with mpsc, so just break)
				break;
			}
			Err(_) => {
				// No more messages
				break;
			}
		}
	}
	(id, path, theme, max_lines)
}

/// Maximum file size to preview (in bytes). Larger files are skipped.
const MAX_PREVIEW_SIZE: u64 = 512 * 1024; // 512 KB

/// Maximum file size for image preview (in bytes). Larger images are skipped.
#[cfg(feature = "media-preview")]
const MAX_IMAGE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

/// Generate syntax-highlighted preview content for a file.
fn generate_preview_impl(
	path: &std::path::Path,
	bat_theme: Option<&str>,
	max_lines: usize,
	assets: &HighlightingAssets,
) -> PreviewContent {
	let path_str = path.display().to_string();

	// Check file metadata
	let metadata = match std::fs::metadata(path) {
		Ok(m) => m,
		Err(e) => return PreviewContent::error(&path_str, format!("Cannot access: {e}")),
	};

	if !metadata.is_file() {
		return PreviewContent::error(&path_str, "Not a file");
	}

	// Handle image files when media-preview is enabled
	#[cfg(feature = "media-preview")]
	if is_image_file(path) {
		if metadata.len() > MAX_IMAGE_SIZE {
			return PreviewContent::error(
				&path_str,
				format!("Image too large ({} MB)", metadata.len() / (1024 * 1024)),
			);
		}

		return match ImagePreview::load(path) {
			Some(image) => PreviewContent::image(&path_str, image),
			None => PreviewContent::error(&path_str, "Failed to load image"),
		};
	}

	if metadata.len() > MAX_PREVIEW_SIZE {
		return PreviewContent::error(
			&path_str,
			format!("File too large ({} KB)", metadata.len() / 1024),
		);
	}

	// Read file content
	let content = match std::fs::read_to_string(path) {
		Ok(c) => c,
		Err(_) => {
			// Try reading as bytes for binary detection
			match std::fs::read(path) {
				Ok(bytes) => {
					if is_binary(&bytes) {
						return PreviewContent::error(&path_str, "Binary file");
					}
					String::from_utf8_lossy(&bytes).into_owned()
				}
				Err(e) => return PreviewContent::error(&path_str, format!("Cannot read: {e}")),
			}
		}
	};

	// Handle empty files
	if content.is_empty() {
		return PreviewContent::empty_file(&path_str);
	}

	// Generate highlighted output using bat
	let highlighted = highlight_with_bat(path, &content, bat_theme, max_lines, assets);

	PreviewContent::text(&path_str, highlighted)
}

/// Check if content appears to be binary.
fn is_binary(bytes: &[u8]) -> bool {
	// Check first 8KB for null bytes (common binary indicator)
	let check_len = bytes.len().min(8192);
	bytes[..check_len].contains(&0)
}

/// Runtime for managing preview generation in the background.
pub struct PreviewRuntime {
	tx: Sender<PreviewCommand>,
	rx: Receiver<PreviewResult>,
	next_id: u64,
	current_id: Option<u64>,
}

impl PreviewRuntime {
	/// Create a new preview runtime with its background worker.
	pub fn new() -> Self {
		let (tx, rx) = spawn();
		Self {
			tx,
			rx,
			next_id: 0,
			current_id: None,
		}
	}

	/// Request a preview for a file. Returns the request ID.
	pub fn request(&mut self, path: PathBuf, theme: Option<String>, max_lines: usize) -> u64 {
		self.next_id = self.next_id.wrapping_add(1);
		let id = self.next_id;
		self.current_id = Some(id);

		let _ = self.tx.send(PreviewCommand::Generate {
			id,
			path,
			theme,
			max_lines,
		});

		id
	}

	/// Try to receive a completed preview result.
	pub fn try_recv(&self) -> Result<PreviewResult, TryRecvError> {
		self.rx.try_recv()
	}

	/// Check if a result matches the most recent request.
	pub fn is_current(&self, id: u64) -> bool {
		self.current_id == Some(id)
	}

	/// Shut down the preview worker.
	pub fn shutdown(&self) {
		let _ = self.tx.send(PreviewCommand::Shutdown);
	}
}

impl Default for PreviewRuntime {
	fn default() -> Self {
		Self::new()
	}
}

impl Drop for PreviewRuntime {
	fn drop(&mut self) {
		self.shutdown();
	}
}
