//! Text selection support for the preview pane, including highlighting,
//! extraction, and clipboard integration.

/// Clipboard helpers (OSC52 and native fallbacks).
pub mod clipboard;
/// Selection text extraction helpers.
pub mod extract;
/// Gutter and wrap accounting for selection.
pub mod gutter;
/// Highlighting helpers for selected spans.
pub mod highlight;
/// Selection state and normalization utilities.
pub mod state;

pub use clipboard::copy_to_clipboard;
pub use extract::extract_selected_text;
pub use highlight::{apply_selection_to_lines, selection_style};
pub use state::TextSelection;

#[cfg(test)]
mod tests;
