use super::file::FileRow;
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
