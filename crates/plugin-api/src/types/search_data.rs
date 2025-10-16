use anyhow::Result;
use ignore::{DirEntry, Error as IgnoreError, WalkBuilder, WalkState};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use std::sync::{Arc, mpsc};

use super::{AttributeRow, FileRow, SearchMode};

/// Data displayed in the search interface, including attributes and files.
#[derive(Debug, Default, Clone)]
pub struct SearchData {
    pub context_label: Option<String>,
    pub initial_query: String,
    pub attributes: Vec<AttributeRow>,
    pub files: Vec<FileRow>,
}

impl SearchData {
    /// Create an empty [`SearchData`] instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a label describing the current search context.
    #[must_use]
    pub fn with_context(mut self, label: impl Into<String>) -> Self {
        self.context_label = Some(label.into());
        self
    }

    /// Set the query that should be shown when the UI starts.
    #[must_use]
    pub fn with_initial_query(mut self, query: impl Into<String>) -> Self {
        self.initial_query = query.into();
        self
    }

    /// Replace the attribute rows with a new collection.
    #[must_use]
    pub fn with_attributes(mut self, attributes: Vec<AttributeRow>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Replace the file rows with a new collection.
    #[must_use]
    pub fn with_files(mut self, files: Vec<FileRow>) -> Self {
        self.files = files;
        self
    }

    /// Build a [`SearchData`] by walking the filesystem under `root`.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying filesystem walker or channel
    /// operations fail while enumerating files.
    pub fn from_filesystem(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let (tx, rx) = mpsc::channel();
        let walker_root = Arc::new(root.clone());
        let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);

        WalkBuilder::new(walker_root.as_path())
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .ignore(true)
            .parents(true)
            .threads(threads)
            .build_parallel()
            .run(|| {
                let sender = tx.clone();
                let root = Arc::clone(&walker_root);
                Box::new(move |entry: Result<DirEntry, IgnoreError>| {
                    if let Ok(entry) = entry {
                        let Some(file_type) = entry.file_type() else {
                            return WalkState::Continue;
                        };
                        if !file_type.is_file() {
                            return WalkState::Continue;
                        }

                        let path = entry.path();
                        let relative = path.strip_prefix(root.as_path()).unwrap_or(path);
                        let tags = tags_for_relative_path(relative);
                        let relative_display = relative.to_string_lossy().replace('\\', "/");
                        let file = FileRow::filesystem(relative_display, tags);
                        if sender.send(file).is_err() {
                            return WalkState::Quit;
                        }
                    }

                    WalkState::Continue
                })
            });

        drop(tx);

        let mut files = Vec::new();
        let mut facet_counts: BTreeMap<String, usize> = BTreeMap::new();

        for file in rx {
            for tag in &file.tags {
                *facet_counts.entry(tag.clone()).or_default() += 1;
            }
            files.push(file);
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));

        let attributes = facet_counts
            .into_iter()
            .map(|(name, count)| AttributeRow::new(name, count))
            .collect();

        Ok(Self {
            context_label: Some(root.display().to_string()),
            initial_query: String::new(),
            attributes,
            files,
        })
    }
}

/// Captures the outcome of a search interaction.
#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub accepted: bool,
    pub selection: Option<SearchSelection>,
    pub query: String,
}

/// The active selection made by the user when a search ends.
#[derive(Debug, Clone)]
pub enum SearchSelection {
    Attribute(AttributeRow),
    File(FileRow),
    Plugin(PluginSelection),
}

/// Selection metadata returned by custom plugins.
#[derive(Debug, Clone)]
pub struct PluginSelection {
    pub mode: SearchMode,
    pub index: usize,
}

impl SearchOutcome {
    /// Return the selected file, if the user confirmed a file result.
    #[must_use]
    pub fn selected_file(&self) -> Option<&FileRow> {
        match self.selection {
            Some(SearchSelection::File(ref file)) => Some(file),
            _ => None,
        }
    }

    /// Return the selected attribute, if the user confirmed a attribute result.
    #[must_use]
    pub fn selected_attribute(&self) -> Option<&AttributeRow> {
        match self.selection {
            Some(SearchSelection::Attribute(ref attribute)) => Some(attribute),
            _ => None,
        }
    }

    /// Return metadata describing a plugin-provided selection.
    #[must_use]
    pub fn selected_plugin(&self) -> Option<&PluginSelection> {
        match self.selection {
            Some(SearchSelection::Plugin(ref plugin)) => Some(plugin),
            _ => None,
        }
    }
}

/// Derive tags for a path relative to the search root.
pub fn tags_for_relative_path(relative: &Path) -> Vec<String> {
    let mut tags: BTreeSet<String> = BTreeSet::new();

    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            if let Component::Normal(part) = component {
                let value = part.to_string_lossy().to_string();
                if !value.is_empty() {
                    tags.insert(value);
                }
            }
        }
    }

    if let Some(ext) = relative.extension().and_then(|ext| ext.to_str())
        && !ext.is_empty()
    {
        tags.insert(format!("*.{ext}"));
    }

    tags.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn builder_methods_replace_data() {
        let attributes = vec![AttributeRow::new("tag", 1)];
        let files = vec![FileRow::new("file", Vec::<String>::new())];
        let data = SearchData::new()
            .with_context("context")
            .with_initial_query("query")
            .with_attributes(attributes.clone())
            .with_files(files.clone());

        assert_eq!(data.context_label.as_deref(), Some("context"));
        assert_eq!(data.initial_query, "query");
        assert_eq!(data.attributes[0].name, "tag");
        assert_eq!(data.files[0].path, "file");
    }

    #[test]
    fn relative_path_tags_include_directories_and_extension() {
        let path = Path::new("dir/sub/file.txt");
        let tags = tags_for_relative_path(path);
        assert_eq!(
            tags,
            vec!["*.txt".to_string(), "dir".to_string(), "sub".to_string()]
        );
    }
}
