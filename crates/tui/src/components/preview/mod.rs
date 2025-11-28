//! File preview pane with syntax highlighting and optional image support.
//!
//! Uses `bat` for text highlighting. With `media-preview` feature, renders
//! images via terminal graphics protocols (Kitty, Sixel, iTerm2, halfblocks).

mod content;
pub(crate) mod highlight;
#[cfg(feature = "media-preview")]
pub mod image;
mod render;
mod worker;
mod wrap;

pub use content::{PreviewContent, PreviewKind};
#[cfg(feature = "media-preview")]
pub use image::{ImagePreview, is_available as is_image_available, is_image_file, protocol_name};
pub use render::{PreviewContext, render_preview};
pub use worker::PreviewRuntime;
pub use wrap::wrap_highlighted_lines;
