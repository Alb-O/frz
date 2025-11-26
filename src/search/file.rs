/// Represents a row in the file results table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileRow {
	/// Stable identifier for this file row, derived from the path.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<u64>,
	/// Filesystem path being represented.
	pub path: String,
	search_text: String,
	truncate: TruncationStyle,
}

impl FileRow {
	/// Build a row for the UI, truncating long paths from the right.
	#[must_use]
	pub fn new(path: impl Into<String>) -> Self {
		Self::from_parts(path.into(), TruncationStyle::Right)
	}

	/// Build a row representing a filesystem entry, truncating from the left.
	#[must_use]
	pub fn filesystem(path: impl Into<String>) -> Self {
		Self::from_parts(path.into(), TruncationStyle::Left)
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

	fn from_parts(path: String, truncate: TruncationStyle) -> Self {
		let search_text = path.clone();
		let id = Some(super::stable_hash64(&path));
		Self {
			id,
			path,
			search_text,
			truncate,
		}
	}
}

/// Controls how a path should be truncated before it is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TruncationStyle {
	/// Truncate from the left side.
	Left,
	/// Truncate from the right side.
	Right,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn filesystem_rows_use_left_truncation() {
		let row = FileRow::filesystem("/tmp/file");
		assert_eq!(row.truncation_style(), TruncationStyle::Left);
	}

	#[test]
	fn search_text_contains_path() {
		let row = FileRow::new("file.txt");
		assert!(row.id.is_some());
		assert_eq!(row.search_text(), "file.txt");
	}
}
