use frz_plugin_api::{
    PluginQueryContext, PluginSelectionContext, SearchData, SearchMode, SearchPlugin,
    SearchSelection, SearchStream,
    descriptors::{
        SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
        TableDescriptor,
    },
    stream_attributes,
};
use frz_tui::tables::rows::build_facet_rows;
use ratatui::layout::{Constraint, Layout, Rect};

const DATASET_KEY: &str = "attributes";

pub fn mode() -> SearchMode {
    SearchMode::from_descriptor(descriptor())
}

pub fn descriptor() -> &'static SearchPluginDescriptor {
    &ATTRIBUTE_DESCRIPTOR
}

static ATTRIBUTE_DATASET: AttributeDataset = AttributeDataset;

pub static ATTRIBUTE_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
    id: DATASET_KEY,
    ui: SearchPluginUiDefinition {
        tab_label: "Tags",
        mode_title: "attribute search",
        hint: "Type to filter attributes. Press Tab to view files.",
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

impl SearchPluginDataset for AttributeDataset {
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
            Some(&column_widths),
        );
        TableDescriptor::new(headers, widths, rows)
    }
}

pub struct AttributeSearchPlugin;

impl SearchPlugin for AttributeSearchPlugin {
    fn descriptor(&self) -> &'static SearchPluginDescriptor {
        descriptor()
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool {
        stream_attributes(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
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
