use ratatui::layout::{Constraint, Layout, Rect};

use crate::extensions::api::{
    Contribution, ExtensionModule, ExtensionPackage, ExtensionQueryContext,
    ExtensionSelectionContext, SearchData, SearchMode, SearchSelection, SearchStream,
    descriptors::{
        ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
    },
    stream_files,
};
use crate::previewers::bat::FilePreviewer;
use crate::tui::tables::rows::build_file_rows;

const DATASET_KEY: &str = "files";

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
        let rows = build_file_rows(
            context.filtered,
            context.scores,
            &context.data.files,
            context.highlight,
            context.highlight_style,
            Some(&column_widths),
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
    contributions: [Contribution; 2],
}

impl FilePackage {
    fn new_contributions() -> [Contribution; 2] {
        [
            Contribution::search_tab(descriptor(), FileModule),
            Contribution::preview_split(descriptor(), FilePreviewer::default()),
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
    type Contributions<'a> = std::array::IntoIter<Contribution, 2>;

    fn contributions(&self) -> Self::Contributions<'_> {
        self.contributions.clone().into_iter()
    }
}

#[must_use]
pub fn bundle() -> FilePackage {
    FilePackage::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_key() {
        assert_eq!(FILE_DATASET.key(), DATASET_KEY);
    }
}
