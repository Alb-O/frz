//! Public-facing filesystem indexing APIs that plugins can opt into using.

pub use super::{
    FilesystemOptions, IndexUpdate, ProgressSnapshot, merge_update, spawn_filesystem_index,
};
