//! Rendering support for the `bat`-powered file preview panel.
//!
//! The module is split into smaller building blocks to make the data flow
//! easier to follow:
//! - `previewer` exposes the [`TextPreviewer`] type that the rest of the
//!   application consumes.
//! - `state` tracks asynchronous preview jobs and caches their output.
//! - `worker` knows how to invoke `bat` with the correct configuration.
//! - `ansi` converts `bat`'s ANSI-coloured output into `ratatui` primitives.
//! - `key` defines the cache key that ties all pieces together.

mod ansi;
mod key;
mod previewer;
mod state;
mod worker;

pub use previewer::TextPreviewer;
