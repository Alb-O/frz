use crate::plugins::{
    PluginQueryContext, PluginSelectionContext, SearchMode, SearchPlugin,
    SearchPluginBehavior, SearchPluginDefinition, SearchPluginUi,
    systems::search::{SearchStream, stream_files},
};
use crate::types::{SearchData, SearchSelection};
use crate::ui::components::tables::{render_table, TablePane};
use crate::ui::App;
use ratatui::layout::Rect;
use ratatui::Frame;

pub(crate) struct FileSearchPlugin;

pub static FILES_DEFINITION: SearchPluginDefinition = SearchPluginDefinition::new(
    "files",
    SearchPluginUi::new(
        "Files",
        "File search",
        "Type to filter files. Press Tab to view facets.",
        "Matching files",
        "Files",
    ),
    SearchPluginBehavior::new(file_dataset_len, render_files),
);

pub const MODE: SearchMode = SearchMode::new(&FILES_DEFINITION);

impl SearchPlugin for FileSearchPlugin {
    fn definition(&self) -> &'static SearchPluginDefinition {
        &FILES_DEFINITION
    }

    fn stream(
        &self,
        query: &str,
        stream: SearchStream<'_>,
        context: PluginQueryContext<'_>,
    ) -> bool {
        stream_files(context.data(), query, stream, context.latest_query_id())
    }

    fn selection(
        &self,
        context: PluginSelectionContext<'_>,
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

fn file_dataset_len(data: &SearchData) -> usize {
    data.files.len()
}

fn render_files(app: &mut App<'_>, frame: &mut Frame<'_>, area: Rect) {
    let highlight_owned = app.highlight_for_query(file_dataset_len(&app.data));
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
        TablePane::Files {
            filtered: &state.filtered,
            scores: &state.scores,
            files: &app.data.files,
            headers: state.headers.as_ref(),
            widths: state.widths.as_ref(),
        },
        &app.theme,
    )
}
