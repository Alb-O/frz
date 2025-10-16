use crate::plugins::{
    PluginQueryContext, PluginSelectionContext, SearchPlugin,
    descriptors::{
        SearchPluginDataset, SearchPluginDescriptor, SearchPluginUiDefinition, TableContext,
        TableDescriptor,
    },
    systems::search::{SearchStream, stream_facets},
};
use crate::types::{SearchData, SearchMode, SearchSelection};
use ratatui::layout::{Constraint, Layout, Rect};

use crate::ui::components::tables::rows::build_facet_rows;

const DATASET_KEY: &str = "facets";

pub fn mode() -> SearchMode {
    SearchMode::from_descriptor(descriptor())
}

pub fn descriptor() -> &'static SearchPluginDescriptor {
    &FACET_DESCRIPTOR
}

static FACET_DATASET: FacetDataset = FacetDataset;

pub(crate) static FACET_DESCRIPTOR: SearchPluginDescriptor = SearchPluginDescriptor {
    id: DATASET_KEY,
    ui: SearchPluginUiDefinition {
        tab_label: "Tags",
        mode_title: "Facet search",
        hint: "Type to filter facets. Press Tab to view files.",
        table_title: "Matching facets",
        count_label: "Facets",
    },
    dataset: &FACET_DATASET,
};

struct FacetDataset;

impl FacetDataset {
    fn default_headers() -> Vec<String> {
        vec!["Facet".into(), "Count".into(), "Score".into()]
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

impl SearchPluginDataset for FacetDataset {
    fn key(&self) -> &'static str {
        DATASET_KEY
    }

    fn total_count(&self, data: &SearchData) -> usize {
        data.facets.len()
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
            &context.data.facets,
            context.highlight,
            Some(&column_widths),
        );
        TableDescriptor::new(headers, widths, rows)
    }
}

pub(crate) struct FacetSearchPlugin;

impl SearchPlugin for FacetSearchPlugin {
    fn descriptor(&self) -> &'static SearchPluginDescriptor {
        descriptor()
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool {
        stream_facets(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
        index: usize,
    ) -> Option<SearchSelection> {
        context
            .data()
            .facets
            .get(index)
            .cloned()
            .map(SearchSelection::Facet)
    }
}
