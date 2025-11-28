//! UI building blocks shared across rendering and state modules.

pub mod preview;
/// Progress tracking and display widget.
pub mod progress;
/// Input prompt rendering and progress display.
pub mod prompt;
/// Table row construction and highlighting.
pub mod rows;
/// Scrollbar for viewports.
pub mod scrollbar;
/// Table rendering and configuration.
pub mod tables;

#[cfg(feature = "media-preview")]
pub use preview::{ImagePreview, PdfPreview, is_image_available, is_pdf_file, protocol_name};
pub use preview::{
	PreviewContent, PreviewContext, PreviewKind, PreviewRuntime, render_preview,
	wrap_highlighted_lines,
};
pub use progress::IndexProgress;
pub use prompt::{InputContext, ProgressState, render_input};
pub use scrollbar::render_scrollbar;
pub use tables::render_table;
