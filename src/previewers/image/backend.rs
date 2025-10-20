//! Terminal graphics capability detection and caching.
//!
//! This module wraps `ratatui-image`'s capability probing so we can run it
//! once during application startup and reuse the result for subsequent image
//! renders. Detecting protocols requires briefly exchanging escape sequences
//! with the terminal, which should only happen after entering the alternate
//! screen and before the main event loop begins. Running the probe once keeps
//! preview rendering lightweight and avoids conflicting with the event reader.

use std::sync::OnceLock;

use anyhow::Result;
use ratatui_image::picker::Picker;

/// Shared graphics state describing the chosen protocol and any warnings that
/// should be surfaced to the user.
pub struct GraphicsBackend {
    picker: Picker,
    warning: Option<String>,
}

impl GraphicsBackend {
    /// Return the cached picker used to build protocol instances for
    /// individual images.
    pub fn picker(&self) -> &Picker {
        &self.picker
    }

    /// Optional warning that explains why protocol detection fell back to a
    /// conservative default.
    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }
}

static GRAPHICS: OnceLock<GraphicsBackend> = OnceLock::new();

/// Probe the running terminal for graphics protocol support.
///
/// This must be called after the terminal enters raw mode and alternate
/// screen, but before event polling starts. Subsequent calls are cheap because
/// the detection result is memoised.
pub fn initialize() -> Result<()> {
    if GRAPHICS.get().is_none() {
        log::debug!(target: "frz::preview::image", "probing terminal graphics capabilities");
        let backend = detect_backend()?;
        let _ = GRAPHICS.set(backend);
    }
    Ok(())
}

/// Retrieve the cached graphics backend, if detection ran successfully.
pub fn backend() -> Option<&'static GraphicsBackend> {
    GRAPHICS.get()
}

fn detect_backend() -> Result<GraphicsBackend> {
    match Picker::from_query_stdio() {
        Ok(picker) => {
            log::debug!(
                target: "frz::preview::image",
                "protocol detected: {:?}, font size {:?}",
                picker.protocol_type(),
                picker.font_size()
            );
            Ok(GraphicsBackend {
                picker,
                warning: None,
            })
        }
        Err(error) => {
            // Fall back to a sensible default that at least provides unicode
            // half-block rendering. `ratatui-image` picks a reasonable
            // character cell size for this scenario, so we can still render an
            // approximate preview while alerting the user that richer
            // protocols were unavailable.
            let picker = Picker::from_fontsize((10, 20));
            let warning = format!(
                "Image protocol detection failed ({error}); falling back to unicode half blocks"
            );
            let backend = GraphicsBackend {
                picker,
                warning: Some(warning),
            };
            log::warn!(
                target: "frz::preview::image",
                "protocol detection failed: {error}"
            );
            log::warn!(
                target: "frz::preview::image",
                "falling back to unicode half-block renderer"
            );
            Ok(backend)
        }
    }
}
