//! File preview pane using the bat library for syntax highlighting.
//!
//! The preview module renders syntax-highlighted file content in a split-view
//! panel, similar to fzf's preview functionality. It uses bat's highlighting
//! engine and theme system which are aligned with frz's built-in themes.
//!
//! Preview generation runs in a background thread to avoid blocking the UI.

mod content;
mod highlight;
mod render;
mod worker;

pub use content::PreviewContent;
pub use render::{PreviewContext, render_preview};
pub use worker::PreviewRuntime;
