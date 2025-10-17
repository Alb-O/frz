use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::attribute::AttributeRow;
use super::file::{FileRow, tags_for_relative_path};
use super::fs::{Fs, OsFs};

/// Data displayed in the search interface, including attributes and files.
#[derive(Debug, Default, Clone)]
pub struct SearchData {
    pub context_label: Option<String>,
    pub root: Option<PathBuf>,
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

    /// Associate the search data with a filesystem root that relative file paths
    /// should be resolved against.
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
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

    /// Resolve a file row to an absolute path on disk when possible.
    #[must_use]
    pub fn resolve_file_path(&self, file: &FileRow) -> PathBuf {
        let candidate = PathBuf::from(&file.path);
        if candidate.is_absolute() {
            return candidate;
        }

        match &self.root {
            Some(root) => root.join(candidate),
            None => candidate,
        }
    }

    /// Build a [`SearchData`] by walking the filesystem under `root`.
    ///
    /// # Errors
    /// Returns an error if the underlying filesystem walker fails while
    /// enumerating files.
    pub fn from_filesystem(root: impl AsRef<Path>) -> Result<Self> {
        Self::from_filesystem_with(&OsFs, root)
    }

    /// Build [`SearchData`] with a caller-provided filesystem implementation.
    ///
    /// Supplying a fake filesystem is useful in tests, where we want
    /// deterministic directory structures without touching the host disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying filesystem walker fails while
    /// enumerating files.
    pub fn from_filesystem_with<F>(fs: &F, root: impl AsRef<Path>) -> Result<Self>
    where
        F: Fs,
    {
        let root = root.as_ref();
        let mut files = Vec::new();
        let mut facet_counts: BTreeMap<String, usize> = BTreeMap::new();

        for entry in fs.walk(root)? {
            let relative = entry?;
            let tags = tags_for_relative_path(relative.as_path());
            let display = relative.to_string_lossy().replace('\\', "/");
            let file = FileRow::filesystem(display, tags);

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
            root: Some(root.to_path_buf()),
            initial_query: String::new(),
            attributes,
            files,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    struct StaticFs {
        entries: Vec<PathBuf>,
    }

    impl StaticFs {
        fn new(entries: &[&str]) -> Self {
            Self {
                entries: entries.iter().map(PathBuf::from).collect(),
            }
        }
    }

    struct StaticIter {
        inner: std::vec::IntoIter<PathBuf>,
    }

    impl Iterator for StaticIter {
        type Item = io::Result<PathBuf>;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next().map(Ok)
        }
    }

    impl Fs for StaticFs {
        type Iter = StaticIter;

        fn walk(&self, _root: &Path) -> io::Result<Self::Iter> {
            Ok(StaticIter {
                inner: self.entries.clone().into_iter(),
            })
        }
    }

    struct HugeFs {
        branching: usize,
        depth: usize,
    }

    impl HugeFs {
        fn new(branching: usize, depth: usize) -> Self {
            Self { branching, depth }
        }
    }

    struct HugeIter {
        branching: usize,
        depth: usize,
        indices: Vec<usize>,
        exhausted: bool,
    }

    impl HugeIter {
        fn new(branching: usize, depth: usize) -> Self {
            let exhausted = depth == 0 || branching == 0;
            Self {
                branching,
                depth,
                indices: vec![0; depth],
                exhausted,
            }
        }

        fn current_path(&self) -> PathBuf {
            if self.depth == 0 {
                return PathBuf::from("placeholder.txt");
            }

            let mut path = PathBuf::new();
            for (level, idx) in self.indices.iter().enumerate() {
                path.push(format!("n{level}_{idx}"));
            }
            path.push("node.txt");
            path
        }

        fn advance(&mut self) {
            if self.depth == 0 || self.branching == 0 {
                self.exhausted = true;
                return;
            }

            for level in (0..self.depth).rev() {
                if self.indices[level] + 1 < self.branching {
                    self.indices[level] += 1;
                    for lower in (level + 1)..self.depth {
                        self.indices[lower] = 0;
                    }
                    return;
                }
            }

            self.exhausted = true;
        }
    }

    impl Iterator for HugeIter {
        type Item = io::Result<PathBuf>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.exhausted {
                return None;
            }

            let path = self.current_path();
            self.advance();
            Some(Ok(path))
        }
    }

    impl Fs for HugeFs {
        type Iter = HugeIter;

        fn walk(&self, _root: &Path) -> io::Result<Self::Iter> {
            Ok(HugeIter::new(self.branching, self.depth))
        }
    }

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
    fn collects_files_from_static_fs() -> anyhow::Result<()> {
        let fs = StaticFs::new(&["a/b.txt", "x/y.rs", "notes.md"]);
        let data = SearchData::from_filesystem_with(&fs, Path::new("/virtual"))?;

        assert_eq!(data.context_label.as_deref(), Some("/virtual"));
        assert_eq!(data.files.len(), 3);
        assert_eq!(data.files[0].path, "a/b.txt");
        assert!(data.attributes.iter().any(|attr| attr.name == "*.txt"));
        assert!(data.attributes.iter().any(|attr| attr.name == "a"));
        Ok(())
    }

    #[test]
    fn walks_tempdir_fixture() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let root = dir.path();

        fs::create_dir_all(root.join("a/b"))?;
        fs::write(root.join("a/b/file.txt"), b"hello")?;
        fs::create_dir_all(root.join("x/y/z"))?;
        fs::write(root.join("x/y/z/another.rs"), b"fn main() {}")?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("a"), root.join("link_to_a"))?;

        let data = SearchData::from_filesystem(root)?;

        assert!(data.files.iter().any(|f| f.path.ends_with("file.txt")));
        assert!(data.attributes.iter().any(|attr| attr.name == "*.txt"));
        assert!(data.attributes.iter().any(|attr| attr.name == "a"));
        Ok(())
    }

    #[test]
    fn synthetic_huge_tree() -> anyhow::Result<()> {
        let fs = HugeFs::new(10, 4);
        let data = SearchData::from_filesystem_with(&fs, Path::new("/"))?;

        assert_eq!(data.files.len(), 10usize.pow(4));
        assert!(data.files.first().unwrap().path.starts_with("n0_0"));
        assert!(data.attributes.iter().any(|attr| attr.name == "*.txt"));
        assert!(data.attributes.iter().any(|attr| attr.name == "n0_0"));
        Ok(())
    }

    #[test]
    fn stress_filesystem() -> anyhow::Result<()> {
        let dir = TempDir::new()?;
        let root = dir.path();

        for d in 0..200 {
            let dir_path = root.join(format!("d{d:04}"));
            fs::create_dir_all(&dir_path)?;
            for f in 0..50 {
                fs::write(dir_path.join(format!("f{f:04}.txt")), b"")?;
            }
        }

        let data = SearchData::from_filesystem(root)?;
        assert!(data.files.len() >= 10_000);
        Ok(())
    }

    #[test]
    fn resolve_file_path_joins_root_for_relative_paths() {
        let data = SearchData::new().with_root("/root");
        let file = FileRow::filesystem("dir/file.txt", Vec::<String>::new());
        let resolved = data.resolve_file_path(&file);
        assert_eq!(resolved, PathBuf::from("/root/dir/file.txt"));
    }

    #[test]
    fn resolve_file_path_preserves_absolute_paths() {
        let data = SearchData::new();
        let file = FileRow::filesystem("/tmp/file.txt", Vec::<String>::new());
        let resolved = data.resolve_file_path(&file);
        assert_eq!(resolved, PathBuf::from("/tmp/file.txt"));
    }
}
