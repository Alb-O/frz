use super::style::StyleConfig;
use crate::tui::theme::types::{Theme, ThemeRegistration};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct ThemeConfig {
    pub(super) name: String,
    #[serde(default)]
    pub(super) aliases: Vec<String>,
    #[serde(default)]
    pub(super) default: bool,
    #[serde(default)]
    pub(super) bat_theme: Option<String>,
    pub(super) styles: ThemeStylesConfig,
}

impl ThemeConfig {
    pub(super) fn into_document(self, context: &str) -> Result<ThemeDocument> {
        let theme = self.styles.into_theme(&format!("{context}.styles"))?;

        let mut registration = ThemeRegistration::new(self.name.clone(), theme);

        if let Some(bat_theme) = self.bat_theme {
            registration = registration.with_bat_theme(bat_theme);
        }

        let registration = self
            .aliases
            .into_iter()
            .map(|alias| alias.trim().to_string())
            .filter(|alias| !alias.is_empty())
            .fold(registration, |registration, alias| {
                registration.alias(alias)
            });

        Ok(ThemeDocument {
            registration,
            is_default: self.default,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ThemeStylesConfig {
    header: StyleConfig,
    row_highlight: StyleConfig,
    prompt: StyleConfig,
    empty: StyleConfig,
    highlight: StyleConfig,
}

impl ThemeStylesConfig {
    fn into_theme(self, context: &str) -> Result<Theme> {
        Ok(Theme {
            header: self.header.to_style(&format!("{context}.header"))?,
            row_highlight: self
                .row_highlight
                .to_style(&format!("{context}.row_highlight"))?,
            prompt: self.prompt.to_style(&format!("{context}.prompt"))?,
            empty: self.empty.to_style(&format!("{context}.empty"))?,
            highlight: self.highlight.to_style(&format!("{context}.highlight"))?,
        })
    }
}

pub(super) struct ThemeDocument {
    pub(super) registration: ThemeRegistration,
    pub(super) is_default: bool,
}
