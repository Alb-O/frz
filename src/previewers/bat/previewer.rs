use std::sync::Mutex;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Clear, Paragraph, Wrap};

use crate::extensions::api::{Icon, PreviewSplit, PreviewSplitContext};

use super::ansi::ansi_to_text;
use super::key::PreviewKey;
use super::state::PreviewState;

/// Renders syntax highlighted previews for the currently selected search result.
#[derive(Default)]
pub struct FilePreviewer {
    state: Mutex<PreviewState>,
}

impl PreviewSplit for FilePreviewer {
    fn render_preview(&self, frame: &mut Frame, area: Rect, context: PreviewSplitContext<'_>) {
        frame.render_widget(Clear, area);

        if area.width == 0 || area.height == 0 {
            return;
        }

        let Some(selected_index) = context.selected_row_index() else {
            render_message(frame, area, "Select a file to preview");
            return;
        };

        let Some(file) = context.data().files.get(selected_index) else {
            render_message(frame, area, "Select a file to preview");
            return;
        };

        let path = context.data().resolve_file_path(file);
        let key = PreviewKey::new(
            path.clone(),
            area.width,
            context.bat_theme(),
            context.git_modifications(),
        );
        let display_path = key.path.display().to_string();

        let mut state = self.state.lock().expect("preview state poisoned");
        state.poll_pending();

        if let Some(result) = state.cached_result(&key) {
            drop(state);
            match result {
                Ok(output) => {
                    let text = ansi_to_text(&output);
                    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
                    frame.render_widget(paragraph, area);
                }
                Err(error) => {
                    let message = format!("Unable to preview {}: {error}", display_path);
                    render_message(frame, area, &message);
                }
            }
            return;
        }

        let mut previous_result = state.cached_output();

        state.ensure_request(key.clone());
        state.poll_pending();

        if let Some(result) = state.cached_result(&key) {
            drop(state);
            match result {
                Ok(output) => {
                    let text = ansi_to_text(&output);
                    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
                    frame.render_widget(paragraph, area);
                }
                Err(error) => {
                    let message = format!("Unable to preview {}: {error}", display_path);
                    render_message(frame, area, &message);
                }
            }
            return;
        }

        if previous_result.is_none() {
            previous_result = state.cached_output();
        }

        drop(state);

        if let Some(Ok(output)) = previous_result {
            let text = ansi_to_text(&output);
            let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
            frame.render_widget(paragraph, area);
            return;
        }

        let message = format!("Loading preview for {}", display_path);
        render_message(frame, area, &message);
    }

    fn header_icon(&self) -> Option<Icon> {
        Some(Icon::from_hex('󰍉', "#61afef"))
    }
}

fn render_message(frame: &mut Frame, area: Rect, message: &str) {
    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(message).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
