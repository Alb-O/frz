//! UI building blocks shared across rendering and state modules.

pub mod preview;
/// Progress tracking and display widget.
pub mod progress;
/// Table row construction and highlighting.
pub mod rows;
/// Table rendering and configuration.
pub mod tables;
/// Tab and input widget components.
pub mod tabs;

pub use preview::{PreviewContent, PreviewContext, PreviewRuntime, render_preview};
pub use progress::IndexProgress;
pub use tables::{TableRenderContext, render_table};
pub use tabs::{InputContext, ProgressState, TabItem, render_input_with_tabs};
