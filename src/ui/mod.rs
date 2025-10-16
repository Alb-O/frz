//! Interactive terminal UI components and helpers.
//!
//! `builder` exposes the public-facing [`SearchUi`] builder, while the other
//! submodules implement the event loop, rendering, and state management that
//! power the terminal application.

mod actions;
mod builder;
mod config;
mod indexing;
mod render;
mod runtime;
mod search;
mod state;

pub use builder::SearchUi;
pub use config::{PaneUiConfig, TabUiConfig, UiConfig};
pub use runtime::run;
pub use state::App;
