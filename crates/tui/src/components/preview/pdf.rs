//! PDF preview via rendering to images and displaying via terminal graphics protocols.
//!
//! PDF files are rendered using Poppler (via `poppler-rs`) and converted to images,
//! then displayed using the same terminal graphics protocols as regular images
//! (Kitty, Sixel, iTerm2, halfblocks).

use std::path::Path;

use cairo::Format;
use image::DynamicImage;
use poppler::Document;

use super::image::{ImagePreview, get_picker};

pub use super::media::is_pdf_file;

/// PDF preview containing rendered first page as an image.
#[derive(Clone, Debug)]
pub struct PdfPreview {
	/// Pre-encoded image of the first page.
	pub image: ImagePreview,
	/// Total number of pages in the PDF.
	pub page_count: u32,
}

impl PdfPreview {
	/// Load and render a PDF file, converting the first page to an image preview.
	pub fn load(path: &Path) -> Result<Self, String> {
		if get_picker().is_none() {
			return Err("No terminal graphics protocol available".to_string());
		}
		let canonical = path
			.canonicalize()
			.map_err(|e| format!("Failed to canonicalize path: {}", e))?;
		let uri = format!("file://{}", canonical.display());
		let document =
			Document::from_file(&uri, None).map_err(|e| format!("Failed to open PDF: {}", e))?;

		let page_count = document.n_pages() as u32;

		if page_count == 0 {
			return Err("PDF has no pages".to_string());
		}

		let page = document
			.page(0)
			.ok_or_else(|| "Failed to get first page".to_string())?;
		let (width, height) = page.size();

		let scale = 150.0 / 72.0; // 72 DPI is default, scale to 150 DPI
		let render_width = (width * scale).ceil() as i32;
		let render_height = (height * scale).ceil() as i32;

		let mut surface = cairo::ImageSurface::create(Format::ARgb32, render_width, render_height)
			.map_err(|e| format!("Failed to create Cairo surface: {}", e))?;

		{
			let context = cairo::Context::new(&surface)
				.map_err(|e| format!("Failed to create Cairo context: {}", e))?;

			// White background
			context.set_source_rgb(1.0, 1.0, 1.0);
			context
				.paint()
				.map_err(|e| format!("Failed to paint background: {}", e))?;

			context.scale(scale, scale);
			page.render(&context);
		}

		let stride = surface.stride() as usize;
		let data = surface
			.data()
			.map_err(|e| format!("Failed to get surface data: {}", e))?;

		let mut rgba_buffer = Vec::with_capacity((render_width * render_height * 4) as usize);

		for y in 0..render_height {
			for x in 0..render_width {
				let offset = (y as usize * stride) + (x as usize * 4);
				if offset + 3 < data.len() {
					// Cairo ARGB32 is stored as BGRA on little-endian -> convert to RGBA
					rgba_buffer.push(data[offset + 2]); // R
					rgba_buffer.push(data[offset + 1]); // G
					rgba_buffer.push(data[offset]); // B
					rgba_buffer.push(data[offset + 3]); // A
				}
			}
		}

		let img =
			image::RgbaImage::from_raw(render_width as u32, render_height as u32, rgba_buffer)
				.ok_or_else(|| "Failed to create image from buffer".to_string())?;
		let dynamic_img = DynamicImage::ImageRgba8(img);

		let image = ImagePreview::from_image(dynamic_img)
			.ok_or_else(|| "Failed to encode image for terminal".to_string())?;

		Ok(Self { image, page_count })
	}

	/// Format page count as a human-readable string (e.g., "1 page" or "5 pages").
	#[must_use]
	pub fn page_count_string(&self) -> String {
		if self.page_count == 1 {
			"1 page".to_string()
		} else {
			format!("{} pages", self.page_count)
		}
	}
}
