use std::path::PathBuf;

/// Identifies a unique preview request produced by the `FilePreviewer`.
///
/// The cache key combines the file path, the available render width and the
/// optional bat theme so we can reuse rendered output whenever the user moves
/// around the UI without re-spawning worker threads.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PreviewKey {
    pub(super) path: PathBuf,
    pub(super) width: u16,
    pub(super) bat_theme: Option<String>,
}

impl PreviewKey {
    /// Creates a new [`PreviewKey`] by normalising the optional theme
    /// reference into an owned [`String`].
    pub(super) fn new(path: PathBuf, width: u16, bat_theme: Option<&str>) -> Self {
        Self {
            path,
            width,
            bat_theme: bat_theme.map(ToString::to_string),
        }
    }
}
