//! Public-facing filesystem indexing APIs that extensions can opt into using.

pub use super::{
    FilesystemOptions, IndexKind, IndexResult, IndexStream, IndexUpdate, ProgressSnapshot,
    merge_update, spawn_filesystem_index,
};
