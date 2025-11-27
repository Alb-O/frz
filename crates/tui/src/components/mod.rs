//! UI building blocks shared across rendering and state modules.

pub mod preview;
/// Progress tracking and display widget.
pub mod progress;
/// Table row construction and highlighting.
pub mod rows;
/// Table rendering and configuration.
pub mod tables;
/// Input widget components.
pub mod tabs;

pub use preview::{PreviewContent, PreviewContext, PreviewKind, PreviewRuntime, render_preview};
pub use progress::IndexProgress;
pub use tables::{TableRenderContext, render_table};
pub use tabs::{InputContext, ProgressState, render_input};

#[cfg(feature = "media-preview")]
pub use preview::{ImagePreview, is_image_available, is_image_file, protocol_name};
