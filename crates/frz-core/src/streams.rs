//! Re-export of the streaming primitives shared across crates.
//!
//! The core streaming types now live in the `frz-stream` crate so they can be
//! reused independently. This module keeps the `crate::streams::*` path stable.

pub use frz_stream::*;
