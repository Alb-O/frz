//! File preview pane with syntax highlighting and optional media support.
//!
//! Uses `bat` for text highlighting. With `media-preview` feature, renders
//! images and PDFs via terminal graphics protocols (Kitty, Sixel, iTerm2, halfblocks).

mod content;
pub(crate) mod highlight;
#[cfg(feature = "media-preview")]
pub mod image;
#[cfg(feature = "media-preview")]
mod media;
#[cfg(feature = "media-preview")]
pub mod pdf;
mod render;
pub mod selection;
mod worker;
mod wrap;

pub use content::{PreviewContent, PreviewKind};
#[cfg(feature = "media-preview")]
pub use image::{ImagePreview, is_available as is_image_available, protocol_name};
#[cfg(feature = "media-preview")]
pub use pdf::{PdfPreview, is_pdf_file};
pub use render::{PreviewContext, render_preview};
pub use selection::{
	TextSelection, apply_selection_to_lines, copy_to_clipboard, extract_selected_text,
	selection_style,
};
pub use worker::PreviewRuntime;
pub use wrap::wrap_highlighted_lines;
