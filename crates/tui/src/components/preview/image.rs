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

use super::media::{center_rect, is_svg_file};

static PICKER: OnceLock<Option<Picker>> = OnceLock::new();

const MAX_SVG_DIMENSION: u32 = 2048;

const DEFAULT_ENCODE_SIZE: Rect = Rect {
	x: 0,
	y: 0,
	width: 80,
	height: 40,
};

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
		.get_or_init(|| picker_from_env().or_else(|| Picker::from_query_stdio().ok()))
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
		let img = if is_svg_file(path) {
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
		let encode_area = encode_size();
		let protocol = picker
			.new_protocol(img, encode_area, Resize::Fit(None))
			.ok()?;

		Some(Self {
			protocol,
			encoded_area: encode_area,
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

fn encode_size() -> Rect {
	static OVERRIDE: OnceLock<Rect> = OnceLock::new();

	*OVERRIDE.get_or_init(|| {
		if let Ok(raw) = std::env::var("FRZ_PREVIEW_IMAGE_ENCODE_CELLS")
			&& let Some((w, h)) = raw.split_once(['x', 'X'])
			&& let (Ok(width), Ok(height)) = (w.trim().parse::<u16>(), h.trim().parse::<u16>())
			&& width > 0
			&& height > 0
		{
			return Rect {
				x: 0,
				y: 0,
				width,
				height,
			};
		}

		DEFAULT_ENCODE_SIZE
	})
}

fn picker_from_env() -> Option<Picker> {
	let requested = std::env::var("FRZ_PREVIEW_IMAGE_PROTOCOL").ok()?;
	let proto = match requested.to_ascii_lowercase().as_str() {
		"halfblocks" | "halfblock" => ProtocolType::Halfblocks,
		"sixel" => ProtocolType::Sixel,
		"kitty" => ProtocolType::Kitty,
		"iterm2" | "iterm" => ProtocolType::Iterm2,
		_ => return None,
	};

	let mut picker = Picker::from_fontsize((8, 16));
	picker.set_protocol_type(proto);
	Some(picker)
}
