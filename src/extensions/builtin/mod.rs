pub mod attributes;
pub mod files;

use crate::extensions::api::SearchMode;

/// Static metadata for a built-in search mode.
#[derive(Clone, Copy)]
pub struct ModeMetadata {
	pub mode: SearchMode,
	pub tab_label: &'static str,
	pub mode_title: &'static str,
	pub hint: &'static str,
	pub table_title: &'static str,
	pub count_label: &'static str,
	pub dataset_key: &'static str,
}

pub fn metadata(mode: SearchMode) -> Option<ModeMetadata> {
	BUILTIN_METADATA
		.iter()
		.find(|meta| meta.mode == mode)
		.copied()
}

pub fn all_metadata() -> &'static [ModeMetadata] {
	&BUILTIN_METADATA
}

const BUILTIN_METADATA: [ModeMetadata; 2] = [
	ModeMetadata {
		mode: SearchMode::Attributes,
		tab_label: "Tags",
		mode_title: "Tag search",
		hint: "Type to filter tags. Press Tab to view files.",
		table_title: "Matching tags",
		count_label: "Tags",
		dataset_key: attributes::DATASET_KEY,
	},
	ModeMetadata {
		mode: SearchMode::Files,
		tab_label: "Files",
		mode_title: "File search",
		hint: "Type to filter files by path or tag. Press Tab to view tags.",
		table_title: "Matching files",
		count_label: "Files",
		dataset_key: files::DATASET_KEY,
	},
];
