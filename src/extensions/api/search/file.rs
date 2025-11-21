use std::collections::BTreeSet;
use std::path::{Component, Path};

/// Represents a row in the file results table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileRow {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<u64>,
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
		let id = Some(super::identity::stable_hash64(&path));
		Self {
			id,
			path,
			tags: tags_sorted,
			display_tags,
			search_text,
			truncate,
		}
	}
}

/// Controls how a path should be truncated before it is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TruncationStyle {
	Left,
	Right,
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

	#[test]
	fn tags_are_sorted_and_displayed() {
		let row = FileRow::new("file.txt", vec!["b", "a"]);
		assert!(row.id.is_some());
		assert_eq!(row.tags, vec!["a", "b"]);
		assert_eq!(row.display_tags, "a, b");
		assert_eq!(row.search_text(), "file.txt a, b");
	}

	#[test]
	fn filesystem_rows_use_left_truncation() {
		let row = FileRow::filesystem("/tmp/file", Vec::<String>::new());
		assert_eq!(row.truncation_style(), TruncationStyle::Left);
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
