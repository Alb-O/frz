use ratatui::layout::{Constraint, Layout, Rect};

use crate::extensions::api::{
    Contribution, ExtensionModule, ExtensionPackage, ExtensionQueryContext,
    ExtensionSelectionContext, SearchData, SearchMode, SearchSelection, SearchStream,
    descriptors::{
        ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
    },
    stream_attributes,
};
use crate::tui::tables::rows::build_facet_rows;

const DATASET_KEY: &str = "attributes";

pub fn mode() -> SearchMode {
    SearchMode::from_descriptor(descriptor())
}

pub fn descriptor() -> &'static ExtensionDescriptor {
    &ATTRIBUTE_DESCRIPTOR
}

static ATTRIBUTE_DATASET: AttributeDataset = AttributeDataset;

pub static ATTRIBUTE_DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
    id: DATASET_KEY,
    ui: ExtensionUiDefinition {
        tab_label: "Tags",
        mode_title: "attribute search",
        hint: "Type to filter attribute.",
        table_title: "Matching attributes",
        count_label: "attributes",
    },
    dataset: &ATTRIBUTE_DATASET,
};

struct AttributeDataset;

impl AttributeDataset {
    fn default_headers() -> Vec<String> {
        vec!["attribute".into(), "Count".into(), "Score".into()]
    }

    fn default_widths() -> Vec<Constraint> {
        vec![
            Constraint::Percentage(50),
            Constraint::Length(8),
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

impl ExtensionDataset for AttributeDataset {
    fn key(&self) -> &'static str {
        DATASET_KEY
    }

    fn total_count(&self, data: &SearchData) -> usize {
        data.attributes.len()
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
        let rows = build_facet_rows(
            context.filtered,
            context.scores,
            &context.data.attributes,
            context.highlight,
            context.highlight_style,
            Some(&column_widths),
        );
        TableDescriptor::new(headers, widths, rows)
    }
}

pub struct AttributeModule;

impl ExtensionModule for AttributeModule {
    fn descriptor(&self) -> &'static ExtensionDescriptor {
        descriptor()
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: ExtensionQueryContext<'_>,
    ) -> bool {
        stream_attributes(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: ExtensionSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection> {
        context
            .data()
            .attributes
            .get(index)
            .cloned()
            .map(SearchSelection::Attribute)
    }
}

pub struct AttributePackage {
    tab: Contribution,
}

impl AttributePackage {
    fn new_tab() -> Contribution {
        Contribution::search_tab(descriptor(), AttributeModule)
    }
}

impl Default for AttributePackage {
    fn default() -> Self {
        Self {
            tab: Self::new_tab(),
        }
    }
}

impl ExtensionPackage for AttributePackage {
    type Contributions<'a>
        = std::iter::Once<Contribution>
    where
        Self: 'a;

    fn contributions(&self) -> Self::Contributions<'_> {
        std::iter::once(self.tab.clone())
    }
}

#[must_use]
pub fn bundle() -> AttributePackage {
    AttributePackage::default()
}
