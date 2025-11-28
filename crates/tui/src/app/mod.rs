//! Core application state and behavior for the interactive picker.
//!
//! The [`App`] type aggregates search data, UI state, and rendering logic.
//! Supporting modules partition the implementation into focused pieces:
//! actions (input handling), rendering, search coordination, and indexing.

mod actions;
mod indexing;
pub(crate) mod preview;
mod render;
mod results;
mod search;
mod state;

pub(crate) use search::SearchRuntime;
pub use state::App;
