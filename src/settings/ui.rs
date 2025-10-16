use anyhow::{Result, anyhow, bail};
use frz::{PaneUiConfig, UiConfig};
use frz_plugin_api::{SearchMode, SearchPluginRegistry};

use super::raw::PaneSection;

/// Create a [`UiConfig`] instance from an optional preset name.
pub(super) fn ui_from_preset(preset: Option<&str>) -> Result<UiConfig> {
    let Some(raw) = preset else {
        return Ok(UiConfig::default());
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(UiConfig::default());
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "default" => Ok(UiConfig::default()),
        "tags-and-files" | "tags_and_files" | "tags" => Ok(UiConfig::tags_and_files()),
        other => bail!("unknown UI preset '{other}'"),
    }
}

/// Apply settings from a [`PaneSection`] onto a mutable [`PaneUiConfig`].
pub(super) fn apply_pane_config(target: &mut PaneUiConfig, pane: PaneSection) {
    if let Some(value) = pane.mode_title {
        target.mode_title = value;
    }
    if let Some(value) = pane.hint {
        target.hint = value;
    }
    if let Some(value) = pane.table_title {
        target.table_title = value;
    }
    if let Some(value) = pane.count_label {
        target.count_label = value;
    }
}

/// Parse a start mode string into a strongly typed [`SearchMode`].
pub(super) fn parse_mode(value: &str) -> Result<SearchMode> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("start mode cannot be empty");
    }
    let id = trimmed.to_ascii_lowercase();
    let mut registry = SearchPluginRegistry::new();
    frz::plugins::builtin::register_builtin_plugins(&mut registry)?;
    registry
        .mode_by_id(&id)
        .or_else(|| registry.mode_by_id(trimmed))
        .ok_or_else(|| anyhow!("unknown start mode '{trimmed}'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use frz::plugins::builtin::{attributes, files};

    #[test]
    fn default_preset_is_returned_for_empty_input() {
        let config = ui_from_preset(Some("   ")).unwrap();
        let default = UiConfig::default();

        assert_eq!(config.filter_label, default.filter_label);
        let attributes_mode = attributes::mode();
        let config_attributes = config.pane(attributes_mode).unwrap();
        let default_attributes = default.pane(attributes_mode).unwrap();
        assert_eq!(config_attributes.mode_title, default_attributes.mode_title);
    }

    #[test]
    fn parse_mode_supports_known_variants() {
        assert_eq!(parse_mode("attributes").unwrap(), attributes::mode());
        assert_eq!(parse_mode("FILES").unwrap(), files::mode());
        assert!(parse_mode("unknown").is_err());
    }

    #[test]
    fn apply_pane_config_overrides_fields() {
        let mut target = PaneUiConfig::new("a", "b", "c", "d");
        let pane = PaneSection {
            mode_title: Some("Mode".into()),
            hint: Some("Hint".into()),
            table_title: Some("Table".into()),
            count_label: Some("Count".into()),
        };

        apply_pane_config(&mut target, pane);

        assert_eq!(target.mode_title, "Mode");
        assert_eq!(target.hint, "Hint");
        assert_eq!(target.table_title, "Table");
        assert_eq!(target.count_label, "Count");
    }
}
