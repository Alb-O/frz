//! Preview content data structures.

use ratatui::text::Line;

/// Cached preview content for a file.
#[derive(Debug, Clone)]
pub struct PreviewContent {
	/// The path that was previewed.
	pub path: String,
	/// Highlighted lines ready for rendering.
	pub lines: Vec<Line<'static>>,
	/// Error message if preview failed.
	pub error: Option<String>,
}

impl PreviewContent {
	/// Create an empty preview (no file selected).
	#[must_use]
	pub fn empty() -> Self {
		Self {
			path: String::new(),
			lines: Vec::new(),
			error: None,
		}
	}

	/// Create an error preview.
	#[must_use]
	pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			lines: Vec::new(),
			error: Some(message.into()),
		}
	}

	/// Create a loading placeholder preview.
	#[must_use]
	pub fn loading(path: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			lines: Vec::new(),
			error: Some("Loading...".into()),
		}
	}

	/// Check if this preview matches a given path.
	#[must_use]
	pub fn matches(&self, path: &str) -> bool {
		self.path == path
	}
}
