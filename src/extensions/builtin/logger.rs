use std::sync::Arc;

use log::LevelFilter;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    widgets::{Cell, Clear, Row},
};

use crate::extensions::api::contributions::{PreviewLayout, PreviewSplit, PreviewSplitContext};
use crate::extensions::api::{
    Contribution, ExtensionModule, ExtensionPackage, ExtensionQueryContext,
    ExtensionSelectionContext, SearchMode, SearchSelection, SearchStream,
    descriptors::{
        ExtensionDataset, ExtensionDescriptor, ExtensionUiDefinition, TableContext, TableDescriptor,
    },
    search::SearchData,
};
use crate::logging;
use crate::tui::theme::Theme;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget, TuiWidgetEvent, TuiWidgetState};

pub const DATASET_KEY: &str = "logs";

pub fn mode() -> SearchMode {
    SearchMode::from_descriptor(descriptor())
}

pub fn descriptor() -> &'static ExtensionDescriptor {
    &LOGGER_DESCRIPTOR
}

static LOGGER_DATASET: LoggerDataset = LoggerDataset;

pub static LOGGER_DESCRIPTOR: ExtensionDescriptor = ExtensionDescriptor {
    id: DATASET_KEY,
    ui: ExtensionUiDefinition {
        tab_label: "Logs",
        mode_title: "Log viewer",
        hint: "Inspect runtime logs and adjust targets with arrow keys.",
        table_title: "Logger controls",
        count_label: "Logs",
    },
    dataset: &LOGGER_DATASET,
};

struct LoggerDataset;

impl ExtensionDataset for LoggerDataset {
    fn key(&self) -> &'static str {
        DATASET_KEY
    }

    fn total_count(&self, _data: &SearchData) -> usize {
        0
    }

    fn build_table<'a>(&self, context: TableContext<'a>) -> TableDescriptor<'a> {
        let headers = context
            .headers
            .cloned()
            .unwrap_or_else(|| vec!["Action".to_string()]);
        let widths = context
            .widths
            .cloned()
            .unwrap_or_else(|| vec![Constraint::Percentage(100)]);
        let rows = vec![Row::new(vec![Cell::from(
            "Use the preview pane to view log output and configure targets.",
        )])];
        TableDescriptor::new(headers, widths, rows)
    }
}

#[derive(Clone)]
struct LoggerModule;

impl ExtensionModule for LoggerModule {
    fn descriptor(&self) -> &'static ExtensionDescriptor {
        descriptor()
    }

    fn stream(
        &self,
        _query: &str,
        stream: SearchStream<'_>,
        _context: ExtensionQueryContext<'_>,
    ) -> bool {
        stream.send(Vec::new(), Vec::new(), true)
    }

    fn selection(
        &self,
        _context: ExtensionSelectionContext<'_>,
        _index: usize,
    ) -> Option<SearchSelection> {
        None
    }
}

pub struct LoggerWidgetState {
    widget: TuiWidgetState,
}

impl LoggerWidgetState {
    pub fn new() -> Self {
        let widget = TuiWidgetState::new().set_default_display_level(LevelFilter::Debug);
        Self { widget }
    }

    pub fn widget(&self) -> &TuiWidgetState {
        &self.widget
    }

    pub fn handle_key(&self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }

        let event = match key.code {
            KeyCode::Char(' ') => Some(TuiWidgetEvent::SpaceKey),
            KeyCode::Char('h') | KeyCode::Char('H') => Some(TuiWidgetEvent::HideKey),
            KeyCode::Char('f') | KeyCode::Char('F') => Some(TuiWidgetEvent::FocusKey),
            KeyCode::Char('+') => Some(TuiWidgetEvent::PlusKey),
            KeyCode::Char('-') => Some(TuiWidgetEvent::MinusKey),
            KeyCode::Up => Some(TuiWidgetEvent::UpKey),
            KeyCode::Down => Some(TuiWidgetEvent::DownKey),
            KeyCode::Left => Some(TuiWidgetEvent::LeftKey),
            KeyCode::Right => Some(TuiWidgetEvent::RightKey),
            KeyCode::PageUp => Some(TuiWidgetEvent::PrevPageKey),
            KeyCode::PageDown => Some(TuiWidgetEvent::NextPageKey),
            KeyCode::Esc => Some(TuiWidgetEvent::EscapeKey),
            _ => None,
        };

        if let Some(event) = event {
            self.widget.transition(event);
            return true;
        }

        false
    }
}

impl Default for LoggerWidgetState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct LoggerPreview {
    state: Arc<LoggerWidgetState>,
}

impl LoggerPreview {
    fn new(state: Arc<LoggerWidgetState>) -> Self {
        Self { state }
    }
}

impl PreviewSplit for LoggerPreview {
    fn render_preview(&self, frame: &mut Frame, area: Rect, _context: PreviewSplitContext<'_>) {
        frame.render_widget(Clear, area);
        if area.width == 0 || area.height == 0 {
            return;
        }

        logging::initialize();
        tui_logger::move_events();

        let widget = TuiLoggerSmartWidget::default()
            .title_log("Runtime log")
            .title_target("Targets")
            .highlight_style(Theme::default().highlight_style())
            .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
            .state(self.state.widget());
        frame.render_widget(widget, area);
    }

    fn layout(&self) -> PreviewLayout {
        PreviewLayout::PreviewOnly
    }

    fn handle_key(&self, key: KeyEvent) -> bool {
        self.state.handle_key(key)
    }
}

pub struct LoggerPackage {
    contributions: [Contribution; 2],
}

impl LoggerPackage {
    fn new() -> Self {
        let state = Arc::new(LoggerWidgetState::new());
        let contributions = [
            Contribution::search_tab(descriptor(), LoggerModule),
            Contribution::preview_split(descriptor(), LoggerPreview::new(state)),
        ];
        Self { contributions }
    }
}

impl Default for LoggerPackage {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionPackage for LoggerPackage {
    type Contributions<'a> = std::array::IntoIter<Contribution, 2>;

    fn contributions(&self) -> Self::Contributions<'_> {
        self.contributions.clone().into_iter()
    }
}

#[must_use]
pub fn bundle() -> LoggerPackage {
    LoggerPackage::default()
}
