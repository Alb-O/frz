/// Represents a row in the file results table.
#[derive(Debug, Clone)]
pub struct FileRow {
    pub path: String,
    pub tags: Vec<String>,
    pub display_tags: String,
    search_text: String,
    truncate: TruncationStyle,
}

impl FileRow {
    /// Build a row for the UI, truncating long paths from the right.
    #[must_use]
    pub fn new<I, S>(path: impl Into<String>, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::from_parts(path.into(), tags, TruncationStyle::Right)
    }

    /// Build a row representing a filesystem entry, truncating from the left.
    #[must_use]
    pub fn filesystem<I, S>(path: impl Into<String>, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::from_parts(path.into(), tags, TruncationStyle::Left)
    }

    /// Return the searchable text composed of the path and display tags.
    pub(crate) fn search_text(&self) -> &str {
        &self.search_text
    }

    /// Return the truncation style to use when rendering the path.
    #[must_use]
    pub fn truncation_style(&self) -> TruncationStyle {
        self.truncate
    }

    fn from_parts<I, S>(path: String, tags: I, truncate: TruncationStyle) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut tags_sorted: Vec<String> = tags.into_iter().map(Into::into).collect();
        tags_sorted.sort();
        let display_tags = tags_sorted.join(", ");
        let search_text = if display_tags.is_empty() {
            path.clone()
        } else {
            format!("{path} {display_tags}")
        };
        Self {
            path,
            tags: tags_sorted,
            display_tags,
            search_text,
            truncate,
        }
    }
}

/// Controls how a path should be truncated before it is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationStyle {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_sorted_and_displayed() {
        let row = FileRow::new("file.txt", vec!["b", "a"]);
        assert_eq!(row.tags, vec!["a", "b"]);
        assert_eq!(row.display_tags, "a, b");
        assert_eq!(row.search_text(), "file.txt a, b");
    }

    #[test]
    fn filesystem_rows_use_left_truncation() {
        let row = FileRow::filesystem("/tmp/file", Vec::<String>::new());
        assert_eq!(row.truncation_style(), TruncationStyle::Left);
    }
}
