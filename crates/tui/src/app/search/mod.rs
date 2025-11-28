//! Search coordination and runtime management.
//!
//! This module handles communication with the background search worker,
//! query sequencing, and result processing.

mod coordination;
mod runtime;

pub(crate) use runtime::SearchRuntime;
