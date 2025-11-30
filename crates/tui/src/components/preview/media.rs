//! Shared media preview infrastructure for images and PDFs.
//!
//! This module provides common utilities for media file preview, using the `infer`
//! crate for magic byte detection with extension-based fallbacks for text formats like SVG.

use std::path::Path;
use std::sync::OnceLock;

use ratatui::layout::Rect;

/// Maximum file size for image preview (in bytes).
pub const MAX_IMAGE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

/// Maximum file size for PDF preview (in bytes).
pub const MAX_PDF_SIZE: u64 = 50 * 1024 * 1024; // 50 MB

/// Detected media file type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
	/// Raster image (PNG, JPEG, GIF, WebP, BMP, etc.)
	Image,
	/// SVG vector image (detected by extension, not magic bytes)
	Svg,
	/// PDF document
	Pdf,
}

/// Detect media type from file content (magic bytes) with extension fallback.
///
/// Returns `None` if the file is not a recognized media type.
#[must_use]
pub fn detect_media_type(path: &Path, buf: &[u8]) -> Option<MediaType> {
	// First try magic byte detection
	if let Some(kind) = infer::get(buf) {
		return match kind.mime_type() {
			"application/pdf" => Some(MediaType::Pdf),
			mime if mime.starts_with("image/") => Some(MediaType::Image),
			_ => None,
		};
	}

	// Fallback to extension for text-based formats (SVG)
	detect_by_extension(path)
}

/// Detect media type from extension only.
///
/// Useful for quick filtering before reading file content.
#[must_use]
pub fn detect_by_extension(path: &Path) -> Option<MediaType> {
	let ext = path.extension()?.to_str()?.to_lowercase();
	match ext.as_str() {
		"svg" => Some(MediaType::Svg),
		"pdf" => Some(MediaType::Pdf),
		"png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tiff" | "tif" | "pnm"
		| "pbm" | "pgm" | "ppm" | "avif" | "heic" | "heif" => Some(MediaType::Image),
		_ => None,
	}
}

/// Max image size with optional env override for constrained environments (e.g., CI/tmux).
#[must_use]
pub fn max_image_size() -> u64 {
	static OVERRIDE: OnceLock<u64> = OnceLock::new();
	*OVERRIDE.get_or_init(|| {
		std::env::var("FRZ_PREVIEW_MAX_IMAGE_BYTES")
			.ok()
			.and_then(|v| v.parse::<u64>().ok())
			.filter(|v| *v > 0)
			.unwrap_or(MAX_IMAGE_SIZE)
	})
}

/// Check if a path might be a PDF file (by extension).
///
/// Use `detect_media_type` with file content for accurate detection.
#[must_use]
pub fn is_pdf_file(path: &Path) -> bool {
	matches!(detect_by_extension(path), Some(MediaType::Pdf))
}

/// Check if a path has an SVG extension.
#[must_use]
pub fn is_svg_file(path: &Path) -> bool {
	matches!(detect_by_extension(path), Some(MediaType::Svg))
}

/// Center a smaller rect within a larger available area.
///
/// This is used by both image and PDF preview rendering to center
/// content within the preview pane.
#[must_use]
pub fn center_rect(inner: Rect, outer: Rect) -> Rect {
	let x = outer.x + (outer.width.saturating_sub(inner.width)) / 2;
	let y = outer.y + (outer.height.saturating_sub(inner.height)) / 2;

	Rect {
		x,
		y,
		width: inner.width.min(outer.width),
		height: inner.height.min(outer.height),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn magic_bytes_override_wrong_extension() {
		// PNG data with .txt extension should still be detected as image
		let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
		assert_eq!(
			detect_media_type(Path::new("fake.txt"), &png_header),
			Some(MediaType::Image)
		);
	}

	#[test]
	fn svg_detected_by_extension_only() {
		// SVG has no magic bytes, relies on extension fallback
		assert_eq!(
			detect_media_type(Path::new("image.svg"), b"<svg"),
			Some(MediaType::Svg)
		);
		// Without .svg extension, XML content isn't detected as SVG
		assert_eq!(detect_media_type(Path::new("image.xml"), b"<svg"), None);
	}

	#[test]
	fn extension_detection_case_insensitive() {
		assert_eq!(
			detect_by_extension(Path::new("test.PNG")),
			Some(MediaType::Image)
		);
		assert_eq!(
			detect_by_extension(Path::new("test.PDF")),
			Some(MediaType::Pdf)
		);
	}

	#[test]
	fn center_rect_clamps_oversized_inner() {
		let inner = Rect::new(0, 0, 30, 20);
		let outer = Rect::new(0, 0, 10, 10);
		let centered = center_rect(inner, outer);

		assert_eq!(centered.width, 10);
		assert_eq!(centered.height, 10);
	}
}
