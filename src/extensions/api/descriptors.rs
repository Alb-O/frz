use frizbee::Options;
use ratatui::{layout::Constraint, style::Style, widgets::Row};

use super::search::SearchData;

/// Static metadata describing a extension contributed to the search experience.
pub struct ExtensionDescriptor {
    /// Stable identifier used to route queries to the extension.
    pub id: &'static str,
    /// UI elements contributed by the extension.
    pub ui: ExtensionUiDefinition,
    /// Dataset definition powering tab rendering and progress accounting.
    pub dataset: &'static dyn ExtensionDataset,
}

/// Declarative description of the UI contributed by a extension.
#[derive(Clone, Copy)]
pub struct ExtensionUiDefinition {
    pub tab_label: &'static str,
    pub mode_title: &'static str,
    pub hint: &'static str,
    pub table_title: &'static str,
    pub count_label: &'static str,
}

/// Information required to render a extension backed table.
#[derive(Clone, Copy)]
pub struct TableContext<'a> {
    pub area: ratatui::layout::Rect,
    pub filtered: &'a [usize],
    pub scores: &'a [u16],
    pub headers: Option<&'a Vec<String>>,
    pub widths: Option<&'a Vec<Constraint>>,
    pub highlight: Option<(&'a str, Options)>,
    pub highlight_style: Style,
    pub selection_width: u16,
    pub column_spacing: u16,
    pub data: &'a SearchData,
}

/// Fully materialized table configuration returned by a dataset implementation.
pub struct TableDescriptor<'a> {
    pub headers: Vec<String>,
    pub widths: Vec<Constraint>,
    pub rows: Vec<Row<'a>>,
}

/// Behavioural definition of a dataset served by a extension.
pub trait ExtensionDataset: Send + Sync {
    /// Stable key describing the dataset. This is used when reporting indexing progress.
    fn key(&self) -> &'static str;

    /// Return the total number of rows available for this dataset.
    fn total_count(&self, data: &SearchData) -> usize;

    /// Render the dataset into a table descriptor tailored to the provided context.
    fn build_table<'a>(&self, context: TableContext<'a>) -> TableDescriptor<'a>;
}

impl<'a> TableDescriptor<'a> {
    /// Convenience constructor for datasets that already own their backing vectors.
    #[must_use]
    pub fn new(headers: Vec<String>, widths: Vec<Constraint>, rows: Vec<Row<'a>>) -> Self {
        Self {
            headers,
            widths,
            rows,
        }
    }
}
