use super::attribute::AttributeRow;
use super::file::FileRow;
use super::mode::SearchMode;

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
    Extension(ExtensionSelection),
}

/// Selection metadata returned by custom extensions.
#[derive(Debug, Clone)]
pub struct ExtensionSelection {
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

    /// Return metadata describing a extension-provided selection.
    #[must_use]
    pub fn selected_extension(&self) -> Option<&ExtensionSelection> {
        match self.selection {
            Some(SearchSelection::Extension(ref extension)) => Some(extension),
            _ => None,
        }
    }
}
