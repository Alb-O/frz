//! Configuration loading and resolution utilities.
//!
//! This module is intentionally decomposed into smaller submodules to keep the
//! configuration pipeline manageable. `load` is the primary entry point and
//! returns a [`ResolvedConfig`] that is used by the application.

mod loader;
mod raw;
mod resolved;
mod sources;
mod ui;
mod util;

pub use loader::load;
pub use resolved::ResolvedConfig;
