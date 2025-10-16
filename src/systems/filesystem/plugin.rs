//! Public-facing filesystem indexing APIs that plugins can opt into using.

pub use super::{
    merge_update,
    spawn_filesystem_index,
    FilesystemOptions,
    IndexUpdate,
    ProgressSnapshot,
};
