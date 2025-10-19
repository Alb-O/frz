use std::path::PathBuf;

use ratatui::{Frame, layout::Rect};

use crate::extensions::api::{
    Icon, PreviewSplit, PreviewSplitContext, contributions::PreviewResource,
};

use super::{bat::TextPreviewer, image::ImagePreviewer};

/// Delegates file previews to either the text or image renderer based on the
/// selected resource.
#[derive(Default)]
pub struct FilePreviewer {
    text: TextPreviewer,
    image: ImagePreviewer,
}

impl FilePreviewer {
    fn resolve_selected_path(context: &PreviewSplitContext<'_>) -> Option<PathBuf> {
        let selection = context.selection()?;
        let PreviewResource::File(file) = selection else {
            return None;
        };
        Some(context.data().resolve_file_path(file))
    }
}

impl PreviewSplit for FilePreviewer {
    fn render_preview(&self, frame: &mut Frame, area: Rect, context: PreviewSplitContext<'_>) {
        if let Some(path) = Self::resolve_selected_path(&context) {
            let display = path.display().to_string();
            if self.image.render(frame, area, &path, &display) {
                return;
            }
        }

        self.text.render_preview(frame, area, context);
    }

    fn header_icon(&self) -> Option<Icon> {
        self.text.header_icon()
    }
}
