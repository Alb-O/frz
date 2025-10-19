use ratatui::layout::{Constraint, Layout, Rect};

use crate::extensions::api::{
    Contribution, ExtensionModule, ExtensionPackage, ExtensionQueryContext,
    ExtensionSelectionContext, Icon, IconProvider, IconResource, IconStore, SearchData, SearchMode,
    SearchSelection, SearchStream,
    contributions::{PreviewResource, SelectionResolver},
    descriptors::{
        ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
    },
    stream_files,
};
use crate::previewers::FilePreviewer;
use crate::tui::tables::rows::build_file_rows;

pub const DATASET_KEY: &str = "files";

pub fn mode() -> SearchMode {
    SearchMode::from_descriptor(descriptor())
}

pub fn descriptor() -> &'static ExtensionDescriptor {
    &FILE_DESCRIPTOR
}

static FILE_DATASET: FileDataset = FileDataset;

pub static FILE_DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
    id: DATASET_KEY,
    ui: ExtensionUiDefinition {
        tab_label: "Files",
        mode_title: "File search",
        hint: "Type to filter files.",
        table_title: "Matching files",
        count_label: "Files",
    },
    dataset: &FILE_DATASET,
};

struct FileDataset;

impl FileDataset {
    fn default_headers() -> Vec<String> {
        vec!["Path".into(), "Tags".into(), "Score".into()]
    }

    fn default_widths() -> Vec<Constraint> {
        vec![
            Constraint::Percentage(60),
            Constraint::Percentage(30),
            Constraint::Length(8),
        ]
    }

    fn resolve_column_widths(
        area: Rect,
        widths: &[Constraint],
        selection_width: u16,
        column_spacing: u16,
    ) -> Vec<u16> {
        if widths.is_empty() {
            return Vec::new();
        }

        let layout_area = Rect {
            x: 0,
            y: 0,
            width: area.width,
            height: 1,
        };
        let [_, columns_area] =
            Layout::horizontal([Constraint::Length(selection_width), Constraint::Fill(0)])
                .areas(layout_area);

        Layout::horizontal(widths.to_vec())
            .spacing(column_spacing)
            .split(columns_area)
            .iter()
            .map(|rect| rect.width)
            .collect()
    }
}

impl ExtensionDataset for FileDataset {
    fn key(&self) -> &'static str {
        DATASET_KEY
    }

    fn total_count(&self, data: &SearchData) -> usize {
        data.files.len()
    }

    fn build_table<'a>(&self, context: TableContext<'a>) -> TableDescriptor<'a> {
        let widths = context.widths.cloned().unwrap_or_else(Self::default_widths);
        let column_widths = Self::resolve_column_widths(
            context.area,
            &widths,
            context.selection_width,
            context.column_spacing,
        );
        let headers = context
            .headers
            .cloned()
            .unwrap_or_else(Self::default_headers);
        let icon_provider = context.scope.resolve::<IconStore>();
        let rows = build_file_rows(
            context.filtered,
            context.scores,
            &context.data.files,
            context.highlight,
            context.highlight_style,
            Some(&column_widths),
            icon_provider,
        );
        TableDescriptor::new(headers, widths, rows)
    }
}

pub struct FileModule;

impl ExtensionModule for FileModule {
    fn descriptor(&self) -> &'static ExtensionDescriptor {
        descriptor()
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: ExtensionQueryContext<'_>,
    ) -> bool {
        stream_files(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: ExtensionSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection> {
        context
            .data()
            .files
            .get(index)
            .cloned()
            .map(SearchSelection::File)
    }
}

pub struct FilePackage {
    contributions: [Contribution; 4],
}

impl FilePackage {
    fn new_contributions() -> [Contribution; 4] {
        [
            Contribution::search_tab(descriptor(), FileModule),
            Contribution::preview_split(descriptor(), FilePreviewer::default()),
            Contribution::icons(descriptor(), FileIcons),
            Contribution::selection_resolver(descriptor(), FileSelectionResolver),
        ]
    }
}

impl Default for FilePackage {
    fn default() -> Self {
        Self {
            contributions: Self::new_contributions(),
        }
    }
}

impl ExtensionPackage for FilePackage {
    type Contributions<'a> = std::array::IntoIter<Contribution, 4>;

    fn contributions(&self) -> Self::Contributions<'_> {
        self.contributions.clone().into_iter()
    }
}

#[must_use]
pub fn bundle() -> FilePackage {
    FilePackage::default()
}

// Use the Font Awesome file glyph for entries without a devicon match.
const GENERIC_FILE_ICON: char = '\u{f016}';

#[derive(Clone, Copy)]
struct FileIcons;

impl IconProvider for FileIcons {
    fn icon_for(&self, resource: IconResource<'_>) -> Option<Icon> {
        match resource {
            IconResource::File(row) => {
                let icon = devicons::FileIcon::from(row.path.as_str());
                let glyph = if icon.icon == '*' {
                    GENERIC_FILE_ICON
                } else {
                    icon.icon
                };
                Some(Icon::from_hex(glyph, icon.color))
            }
        }
    }
}

#[derive(Clone, Copy)]
struct FileSelectionResolver;

impl SelectionResolver for FileSelectionResolver {
    fn resolve<'a>(
        &self,
        data: &'a SearchData,
        filtered: &'a [usize],
        selected: Option<usize>,
    ) -> Option<PreviewResource<'a>> {
        let index = selected?;
        let row_index = filtered.get(index).copied()?;
        data.files.get(row_index).map(PreviewResource::File)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::api::search::FileRow;

    #[test]
    fn dataset_key() {
        assert_eq!(FILE_DATASET.key(), DATASET_KEY);
    }

    #[test]
    fn unknown_extensions_use_generic_file_icon() {
        let provider = FileIcons;
        let row = FileRow::filesystem("file.zzz", Vec::<String>::new());

        let icon = provider
            .icon_for(IconResource::File(&row))
            .expect("file icons should exist");

        assert_eq!(icon, Icon::from_hex(GENERIC_FILE_ICON, "#7e8ea8"));
    }
}
