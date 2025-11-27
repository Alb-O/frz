//! Image preview via terminal graphics protocols (Kitty, Sixel, iTerm2, halfblocks).
//!
//! SVG files are rasterized with `resvg` before display.
//!
//! Images are pre-encoded in the background worker thread using `Picker::new_protocol()`
//! so that rendering is instant and doesn't block the UI thread.

use std::path::Path;
use std::sync::OnceLock;

use image::{DynamicImage, RgbaImage};
use ratatui::layout::Rect;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};

static PICKER: OnceLock<Option<Picker>> = OnceLock::new();

const IMAGE_EXTENSIONS: &[&str] = &[
	"png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "tiff", "tif", "pnm", "pbm", "pgm", "ppm",
	"svg",
];

const MAX_SVG_DIMENSION: u32 = 2048;

/// Default size for pre-encoding images (in terminal cells).
/// This should be large enough for most preview panes.
const DEFAULT_ENCODE_SIZE: Rect = Rect {
	x: 0,
	y: 0,
	width: 80,
	height: 40,
};

/// Check if a path has a recognized image extension.
#[must_use]
pub fn is_image_file(path: &Path) -> bool {
	path.extension()
		.and_then(|ext| ext.to_str())
		.is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

fn is_svg(path: &Path) -> bool {
	path.extension()
		.and_then(|ext| ext.to_str())
		.is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

fn render_svg(path: &Path) -> Option<DynamicImage> {
	let data = std::fs::read(path).ok()?;
	let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default()).ok()?;
	let size = tree.size();

	let scale =
		if size.width() > MAX_SVG_DIMENSION as f32 || size.height() > MAX_SVG_DIMENSION as f32 {
			(MAX_SVG_DIMENSION as f32 / size.width()).min(MAX_SVG_DIMENSION as f32 / size.height())
		} else {
			1.0
		};

	let (width, height) = (
		(size.width() * scale).ceil() as u32,
		(size.height() * scale).ceil() as u32,
	);

	let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;
	resvg::render(
		&tree,
		resvg::tiny_skia::Transform::from_scale(scale, scale),
		&mut pixmap.as_mut(),
	);

	RgbaImage::from_raw(width, height, pixmap.take()).map(DynamicImage::ImageRgba8)
}

/// Get the global image picker (lazily initialized).
pub fn get_picker() -> Option<&'static Picker> {
	PICKER
		.get_or_init(|| Picker::from_query_stdio().ok())
		.as_ref()
}

/// Check if image preview is available in the current terminal.
#[must_use]
pub fn is_available() -> bool {
	get_picker().is_some()
}

/// Get the name of the detected graphics protocol.
#[must_use]
pub fn protocol_name() -> &'static str {
	get_picker()
		.map(|p| match p.protocol_type() {
			ProtocolType::Kitty => "Kitty",
			ProtocolType::Sixel => "Sixel",
			ProtocolType::Iterm2 => "iTerm2",
			ProtocolType::Halfblocks => "Halfblocks",
		})
		.unwrap_or("None")
}

/// Pre-encoded image ready for instant terminal rendering.
///
/// The image is encoded once during loading (in a background thread),
/// so rendering is non-blocking.
#[derive(Clone)]
pub struct ImagePreview {
	/// Pre-encoded protocol data for instant rendering.
	protocol: Protocol,
	/// The area the image was encoded for.
	encoded_area: Rect,
	/// Image dimensions in pixels (width, height).
	pub dimensions: (u32, u32),
}

impl std::fmt::Debug for ImagePreview {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ImagePreview")
			.field("dimensions", &self.dimensions)
			.field("encoded_area", &self.encoded_area)
			.finish_non_exhaustive()
	}
}

impl ImagePreview {
	/// Load and pre-encode an image from a file path.
	///
	/// This performs the expensive encoding work upfront so that
	/// rendering is instant.
	pub fn load(path: &Path) -> Option<Self> {
		let picker = get_picker()?;
		let img = if is_svg(path) {
			render_svg(path)?
		} else {
			image::ImageReader::open(path).ok()?.decode().ok()?
		};
		Self::from_image_with_picker(img, picker)
	}

	/// Create from an already-decoded image.
	pub fn from_image(img: DynamicImage) -> Option<Self> {
		let picker = get_picker()?;
		Self::from_image_with_picker(img, picker)
	}

	fn from_image_with_picker(img: DynamicImage, picker: &Picker) -> Option<Self> {
		let dimensions = (img.width(), img.height());

		// Pre-encode for a reasonable preview size
		let protocol = picker
			.new_protocol(img, DEFAULT_ENCODE_SIZE, Resize::Fit(None))
			.ok()?;

		Some(Self {
			protocol,
			encoded_area: DEFAULT_ENCODE_SIZE,
			dimensions,
		})
	}

	/// Render the image centered within the available area.
	///
	/// This is instant because the image was pre-encoded during loading.
	pub fn render(&self, frame: &mut ratatui::Frame, area: Rect) {
		// Get the area the protocol was encoded for
		let image_area = self.protocol.area();

		// Center the image within the available area
		let centered = center_rect(image_area, area);

		// Render using the stateless Image widget (instant, no encoding)
		let widget = Image::new(&self.protocol);
		frame.render_widget(widget, centered);
	}

	/// Format dimensions as "W×H".
	#[must_use]
	pub fn dimensions_string(&self) -> String {
		format!("{}×{}", self.dimensions.0, self.dimensions.1)
	}
}

/// Center a smaller rect within a larger available area.
fn center_rect(inner: Rect, outer: Rect) -> Rect {
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
	fn test_is_image_file() {
		assert!(is_image_file(Path::new("test.png")));
		assert!(is_image_file(Path::new("test.PNG")));
		assert!(is_image_file(Path::new("test.jpg")));
		assert!(is_image_file(Path::new("test.gif")));
		assert!(is_image_file(Path::new("test.svg")));
		assert!(is_image_file(Path::new("/path/to/image.bmp")));

		assert!(!is_image_file(Path::new("test.rs")));
		assert!(!is_image_file(Path::new("test.txt")));
		assert!(!is_image_file(Path::new("test")));
	}

	#[test]
	fn test_center_rect() {
		let inner = Rect::new(0, 0, 10, 5);
		let outer = Rect::new(0, 0, 20, 10);
		let centered = center_rect(inner, outer);

		assert_eq!(centered.x, 5);
		assert_eq!(centered.y, 2);
		assert_eq!(centered.width, 10);
		assert_eq!(centered.height, 5);
	}
}
