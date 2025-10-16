//! Long-lived background systems that plugins can opt into.

#[cfg(feature = "fs")]
pub mod filesystem;
pub mod search;
