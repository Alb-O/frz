//! Interactive terminal UI orchestration for `frz`.
//!
//! This feature module contains the full TUI application including the builder,
//! event loop, rendering pipeline, state management, and the reusable widgets/style
//! definitions that power the terminal application.

mod app;
mod builder;
pub mod components;
mod config;
/// Syntax highlighting and text styling utilities.
pub mod highlight;
pub mod input;
mod runtime;
pub mod style;

pub use app::App;
pub use builder::Picker;
pub use config::{PaneLabels, TabLabels, UiLabels};
pub use runtime::run;

pub use crate::components::{progress, prompt, rows as utils, tables};
pub use crate::input::QueryInput;
pub use crate::style::{StyleConfig, Theme, builtin_themes, default_theme};
