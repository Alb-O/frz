use super::file::FileRow;
/// Captures the outcome of a search interaction.
#[derive(Debug, Clone)]
pub struct SearchOutcome {
	/// Whether the user confirmed the selection.
	pub accepted: bool,
	/// The selected item, if any.
	pub selection: Option<SearchSelection>,
	/// The query string that was active.
	pub query: String,
}

/// The active selection made by the user when a search ends.
#[derive(Debug, Clone)]
pub enum SearchSelection {
	/// A file was selected.
	File(FileRow),
}

impl SearchOutcome {
	/// Return the selected file, if the user confirmed a file result.
	#[must_use]
	pub fn selected_file(&self) -> Option<&FileRow> {
		match self.selection {
			Some(SearchSelection::File(ref file)) => Some(file),
			None => None,
		}
	}
}
