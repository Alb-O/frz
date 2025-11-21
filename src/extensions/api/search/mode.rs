/// Identifies the built-in search tabs supported by the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchMode {
	Attributes,
	Files,
}

impl SearchMode {
	/// Stable string identifier for the mode, used in configuration and UI labels.
	#[must_use]
	pub const fn id(self) -> &'static str {
		match self {
			SearchMode::Attributes => "attributes",
			SearchMode::Files => "files",
		}
	}

	/// List of all supported modes in their default order.
	#[must_use]
	pub const fn all() -> [SearchMode; 2] {
		[SearchMode::Attributes, SearchMode::Files]
	}
}
