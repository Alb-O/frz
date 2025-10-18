use std::collections::HashMap;
use std::sync::Arc;

use ratatui::style::{Color, Style};
use ratatui::text::Span;

use crate::extensions::api::descriptors::ExtensionDescriptor;
use crate::extensions::api::error::ExtensionCatalogError;
use crate::extensions::api::search::{FileRow, SearchMode};

use super::{ContributionInstallContext, ContributionSpecImpl, ScopedContribution};

/// Icon rendered alongside a table row contribution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Icon {
    glyph: char,
    color: Option<Color>,
}

impl Icon {
    /// Create a new icon with an optional foreground color.
    #[must_use]
    pub fn new(glyph: char, color: Option<Color>) -> Self {
        Self { glyph, color }
    }

    /// Create an icon from a glyph and hexadecimal colour code.
    #[must_use]
    pub fn from_hex(glyph: char, hex: &str) -> Self {
        Self {
            glyph,
            color: parse_hex_color(hex),
        }
    }

    /// Return the styled span representing this icon with a trailing space.
    ///
    /// Padding the icon ensures double-width glyphs keep their visual size because the
    /// terminal will not overwrite the second cell with a separate span containing a
    /// space character. This keeps icons consistently sized regardless of focus state.
    #[must_use]
    pub fn to_padded_span(self) -> Span<'static> {
        let mut text = String::new();
        text.push(self.glyph);
        text.push(' ');
        let style = self
            .color
            .map_or_else(Style::default, |color| Style::default().fg(color));
        Span::styled(text, style)
    }
}

/// Context describing the resource a icon is requested for.
#[derive(Clone, Copy)]
pub enum IconResource<'a> {
    File(&'a FileRow),
}

/// Providers that supply icons for rows rendered by an extension.
pub trait IconProvider: Send + Sync {
    fn icon_for(&self, resource: IconResource<'_>) -> Option<Icon>;
}

#[derive(Clone, Default)]
pub struct IconStore {
    providers: HashMap<SearchMode, Arc<dyn IconProvider>>,
}

impl IconStore {
    pub fn register(
        &mut self,
        mode: SearchMode,
        provider: Arc<dyn IconProvider>,
    ) -> Result<(), ExtensionCatalogError> {
        if self.providers.contains_key(&mode) {
            return Err(ExtensionCatalogError::contribution_conflict("icons", mode));
        }
        self.providers.insert(mode, provider);
        Ok(())
    }

    pub fn get(&self, mode: SearchMode) -> Option<Arc<dyn IconProvider>> {
        self.providers.get(&mode).cloned()
    }

    pub fn remove(&mut self, mode: SearchMode) {
        self.providers.remove(&mode);
    }
}

impl ScopedContribution for IconStore {
    type Output = Arc<dyn IconProvider>;

    fn resolve(&self, mode: SearchMode) -> Option<Self::Output> {
        self.get(mode)
    }
}

/// Contribution describing a icon provider for a search mode.
#[derive(Clone)]
pub struct IconContribution {
    descriptor: &'static ExtensionDescriptor,
    provider: Arc<dyn IconProvider>,
}

impl IconContribution {
    pub fn new<P>(descriptor: &'static ExtensionDescriptor, provider: P) -> Self
    where
        P: IconProvider + 'static,
    {
        let provider: Arc<dyn IconProvider> = Arc::new(provider);
        Self {
            descriptor,
            provider,
        }
    }
}

impl ContributionSpecImpl for IconContribution {
    fn install(
        &self,
        context: &mut ContributionInstallContext<'_>,
    ) -> Result<(), ExtensionCatalogError> {
        let mode = SearchMode::from_descriptor(self.descriptor);
        let store = context.storage_mut::<IconStore>();
        store.register(mode, Arc::clone(&self.provider))?;
        context.register_cleanup::<IconStore, _>(IconStore::remove);
        Ok(())
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(red, green, blue))
}
