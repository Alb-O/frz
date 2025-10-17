//! Terminal UI building blocks for rendering `frz`.
//!
//! The submodules here expose reusable widgets, input helpers, and supporting
//! utilities used by the higher level UI orchestration code.

pub mod components;
pub mod highlight;
pub mod input;
pub mod tables;
pub mod theme;

pub use highlight::highlight_cell;
