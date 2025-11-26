//! Interactive terminal UI orchestration for `frz`.
//!
//! The [`builder`] module exposes the public-facing [`SearchUi`] builder. The
//! remaining submodules implement the event loop, rendering pipeline, state
//! management, and the reusable widgets/style definitions that power the
//! terminal application.

mod actions;
mod builder;
pub mod components;
mod config;
/// Syntax highlighting and text styling utilities.
pub mod highlight;
mod indexing;
pub mod input;
/// Terminal frame rendering orchestration.
mod render;
mod runtime;
mod search;
mod state;
pub mod style;

pub use builder::SearchUi;
pub use config::{PaneUiConfig, TabUiConfig, UiConfig};
pub use runtime::run;
pub use state::App;
