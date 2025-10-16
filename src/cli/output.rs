#[cfg(feature = "fs")]
use anyhow::Result;
#[cfg(feature = "fs")]
use serde_json::json;

#[cfg(feature = "fs")]
use frz::types::{PluginSelection, SearchOutcome, SearchSelection};

#[cfg(feature = "fs")]
/// Print a plain-text representation of the search outcome.
pub(crate) fn print_plain(outcome: &SearchOutcome) {
    if !outcome.accepted {
        println!("Search cancelled (query: '{}')", outcome.query);
        return;
    }

    match &outcome.selection {
        Some(SearchSelection::File(file)) => println!("{}", file.path),
        Some(SearchSelection::Facet(facet)) => println!("Facet: {}", facet.name),
        Some(SearchSelection::Plugin(plugin)) => {
            println!("Plugin selection: {} @ {}", plugin.mode.id(), plugin.index)
        }
        None => println!("No selection"),
    }
}

#[cfg(feature = "fs")]
/// Format the search outcome as a JSON string.
pub(crate) fn format_outcome_json(outcome: &SearchOutcome) -> Result<String> {
    let selection = match &outcome.selection {
        Some(SearchSelection::File(file)) => json!({
            "type": "file",
            "path": file.path,
            "tags": file.tags,
            "display_tags": file.display_tags,
        }),
        Some(SearchSelection::Facet(facet)) => json!({
            "type": "facet",
            "name": facet.name,
            "count": facet.count,
        }),
        Some(SearchSelection::Plugin(PluginSelection { mode, index })) => json!({
            "type": "plugin",
            "mode": mode.id(),
            "index": index,
        }),
        None => serde_json::Value::Null,
    };

    let payload = json!({
        "accepted": outcome.accepted,
        "query": outcome.query,
        "selection": selection,
    });

    Ok(serde_json::to_string_pretty(&payload)?)
}

#[cfg(feature = "fs")]
/// Print the JSON representation of the search outcome.
pub(crate) fn print_json(outcome: &SearchOutcome) -> Result<()> {
    println!("{}", format_outcome_json(outcome)?);
    Ok(())
}

#[cfg(all(test, feature = "fs"))]
mod tests {
    use super::*;
    use frz::plugins::builtin::files;
    use frz::types::{FacetRow, FileRow};
    use serde_json::Value;

    #[test]
    fn json_format_includes_file_selection() {
        let outcome = SearchOutcome {
            accepted: true,
            query: "test".into(),
            selection: Some(SearchSelection::File(FileRow::new("path", ["a"]))),
        };

        let json = format_outcome_json(&outcome).expect("json");
        let value: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(value["selection"]["type"], "file");
        assert_eq!(value["selection"]["path"], "path");
        assert_eq!(value["selection"]["tags"][0], "a");
    }

    #[test]
    fn json_format_includes_facet_selection() {
        let outcome = SearchOutcome {
            accepted: true,
            query: "test".into(),
            selection: Some(SearchSelection::Facet(FacetRow::new("Facet", 3))),
        };

        let json = format_outcome_json(&outcome).expect("json");
        let value: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(value["selection"]["type"], "facet");
        assert_eq!(value["selection"]["name"], "Facet");
        assert_eq!(value["selection"]["count"], 3);
    }

    #[test]
    fn json_format_includes_plugin_selection() {
        let outcome = SearchOutcome {
            accepted: true,
            query: "test".into(),
            selection: Some(SearchSelection::Plugin(PluginSelection {
                mode: files::mode(),
                index: 7,
            })),
        };

        let json = format_outcome_json(&outcome).expect("json");
        let value: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(value["selection"]["type"], "plugin");
        assert_eq!(value["selection"]["mode"], "files");
        assert_eq!(value["selection"]["index"], 7);
    }
}
