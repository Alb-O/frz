use crate::plugins::{
    PluginQueryContext, PluginSelectionContext, SearchMode, SearchPlugin,
    SearchPluginBehavior, SearchPluginDefinition, SearchPluginUi,
    systems::search::{SearchStream, stream_facets},
};
use crate::types::{SearchData, SearchSelection};
use crate::ui::components::tables::{render_table, TablePane};
use crate::ui::App;
use ratatui::layout::Rect;
use ratatui::Frame;

pub(crate) struct FacetSearchPlugin;

pub static FACETS_DEFINITION: SearchPluginDefinition = SearchPluginDefinition::new(
    "facets",
    SearchPluginUi::new(
        "Tags",
        "Facet search",
        "Type to filter facets. Press Tab to view files.",
        "Matching facets",
        "Facets",
    ),
    SearchPluginBehavior::new(facet_dataset_len, render_facets),
);

pub const MODE: SearchMode = SearchMode::new(&FACETS_DEFINITION);

impl SearchPlugin for FacetSearchPlugin {
    fn definition(&self) -> &'static SearchPluginDefinition {
        &FACETS_DEFINITION
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

fn facet_dataset_len(data: &SearchData) -> usize {
    data.facets.len()
}

fn render_facets(app: &mut App<'_>, frame: &mut Frame<'_>, area: Rect) {
    let highlight_owned = app.highlight_for_query(facet_dataset_len(&app.data));
    let highlight_state = highlight_owned
        .as_ref()
        .map(|(text, config)| (text.as_str(), *config));
    let state = app.tab_states.entry(MODE).or_default();
    render_table(
        frame,
        area,
        &mut app.table_state,
        &app.ui,
        highlight_state,
        TablePane::Facets {
            filtered: &state.filtered,
            scores: &state.scores,
            facets: &app.data.facets,
            headers: state.headers.as_ref(),
            widths: state.widths.as_ref(),
        },
        &app.theme,
    )
}
