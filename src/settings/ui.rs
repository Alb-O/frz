use anyhow::{Result, bail};
use frz::plugins::builtin::{FACETS_DEFINITION, FILES_DEFINITION, FACETS_MODE, FILES_MODE};
use frz::{PaneUiConfig, SearchMode, UiConfig};

use super::raw::PaneSection;

/// Create a [`UiConfig`] instance from an optional preset name.
pub(super) fn ui_from_preset(preset: Option<&str>) -> Result<UiConfig> {
    let default = UiConfig::for_definitions([&FACETS_DEFINITION, &FILES_DEFINITION]);
    let Some(raw) = preset else {
        return Ok(default);
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(default);
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "default" | "tags-and-files" | "tags_and_files" | "tags" => Ok(default),
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
    match value.trim().to_ascii_lowercase().as_str() {
        "facets" => Ok(FACETS_MODE),
        "files" => Ok(FILES_MODE),
        other => bail!("unknown start mode '{other}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preset_is_returned_for_empty_input() {
        let config = ui_from_preset(Some("   ")).unwrap();
        let default = UiConfig::for_definitions([&FACETS_DEFINITION, &FILES_DEFINITION]);

        assert_eq!(config.filter_label, default.filter_label);
        let config_facets = config.pane(FACETS_MODE).unwrap();
        let default_facets = default.pane(FACETS_MODE).unwrap();
        assert_eq!(config_facets.mode_title, default_facets.mode_title);
    }

    #[test]
    fn parse_mode_supports_known_variants() {
        assert_eq!(parse_mode("facets").unwrap(), FACETS_MODE);
        assert_eq!(parse_mode("FILES").unwrap(), FILES_MODE);
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
