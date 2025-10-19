//! Application state management for the interactive terminal UI.
//!
//! The [`App`] type aggregates search data, extension metadata, and the
//! rendering state shared across panes. Supporting modules break the logic into
//! smaller, focused pieces so that individual concerns can evolve
//! independently.

mod app;
mod search_runtime;

pub use app::App;
pub(crate) use search_runtime::SearchRuntime;
