//! Preview content data structures.

use ratatui::text::Line;

#[cfg(feature = "media-preview")]
use super::image::ImagePreview;
#[cfg(feature = "media-preview")]
use super::pdf::PdfPreview;

/// The type of content being previewed.
#[derive(Debug, Clone)]
pub enum PreviewKind {
	/// Syntax-highlighted text content.
	Text {
		/// Highlighted lines.
		lines: Vec<Line<'static>>,
	},
	/// Image content (requires `media-preview` feature).
	#[cfg(feature = "media-preview")]
	Image {
		/// Loaded image.
		image: ImagePreview,
	},
	/// PDF content (requires `media-preview` feature).
	#[cfg(feature = "media-preview")]
	Pdf {
		/// Rendered PDF preview.
		pdf: PdfPreview,
	},
	/// Placeholder (loading, error, empty).
	Placeholder {
		/// Message to display.
		message: String,
	},
}

/// Cached preview content for a file.
#[derive(Debug, Clone)]
pub struct PreviewContent {
	/// Path of the previewed file.
	pub path: String,
	/// Content kind.
	pub kind: PreviewKind,
}

impl PreviewContent {
	/// Empty preview (no file selected).
	#[must_use]
	pub fn empty() -> Self {
		Self {
			path: String::new(),
			kind: PreviewKind::Placeholder {
				message: String::new(),
			},
		}
	}

	/// Preview for an empty file.
	#[must_use]
	pub fn empty_file(path: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			kind: PreviewKind::Placeholder {
				message: "Empty file".into(),
			},
		}
	}

	/// Error preview.
	#[must_use]
	pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			kind: PreviewKind::Placeholder {
				message: message.into(),
			},
		}
	}

	/// Loading placeholder.
	#[must_use]
	pub fn loading(path: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			kind: PreviewKind::Placeholder {
				message: "Loading...".into(),
			},
		}
	}

	/// Text preview with highlighted lines.
	#[must_use]
	pub fn text(path: impl Into<String>, lines: Vec<Line<'static>>) -> Self {
		Self {
			path: path.into(),
			kind: PreviewKind::Text { lines },
		}
	}

	/// Image preview.
	#[cfg(feature = "media-preview")]
	#[must_use]
	pub fn image(path: impl Into<String>, image: ImagePreview) -> Self {
		Self {
			path: path.into(),
			kind: PreviewKind::Image { image },
		}
	}

	/// PDF preview.
	#[cfg(feature = "media-preview")]
	#[must_use]
	pub fn pdf(path: impl Into<String>, pdf: PdfPreview) -> Self {
		Self {
			path: path.into(),
			kind: PreviewKind::Pdf { pdf },
		}
	}

	/// Check if this preview matches a path.
	#[must_use]
	pub fn matches(&self, path: &str) -> bool {
		self.path == path
	}

	/// Check if this is a placeholder.
	#[must_use]
	pub fn is_placeholder(&self) -> bool {
		matches!(self.kind, PreviewKind::Placeholder { .. })
	}

	/// Get the placeholder message if any.
	#[must_use]
	pub fn error_message(&self) -> Option<&str> {
		match &self.kind {
			PreviewKind::Placeholder { message } if !message.is_empty() => Some(message),
			_ => None,
		}
	}

	/// Get text lines if this is a text preview.
	#[must_use]
	pub fn lines(&self) -> Option<&[Line<'static>]> {
		match &self.kind {
			PreviewKind::Text { lines } => Some(lines),
			_ => None,
		}
	}

	/// Number of text lines (0 for non-text content).
	#[must_use]
	pub fn line_count(&self) -> usize {
		self.lines().map_or(0, |l| l.len())
	}
}
