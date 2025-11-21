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
pub mod highlight;
mod indexing;
pub mod input;
pub mod render;
mod runtime;
mod search;
mod state;
pub mod style;

pub use builder::SearchUi;
pub use config::{PaneUiConfig, TabUiConfig, UiConfig};
pub use runtime::run;
pub use state::App;
