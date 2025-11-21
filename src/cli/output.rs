use anyhow::Result;
use frz::{SearchOutcome, SearchSelection};
use serde_json::json;

/// Print a plain-text representation of the search outcome.
pub(crate) fn print_plain(outcome: &SearchOutcome) {
	if !outcome.accepted {
		println!("Search cancelled (query: '{}')", outcome.query);
		return;
	}

	match &outcome.selection {
		Some(SearchSelection::File(file)) => println!("{}", file.path),
		Some(SearchSelection::Attribute(attribute)) => println!("attribute: {}", attribute.name),
		None => println!("No selection"),
	}
}

/// Format the search outcome as a JSON string.
pub(crate) fn format_outcome_json(outcome: &SearchOutcome) -> Result<String> {
	let selection = match &outcome.selection {
		Some(SearchSelection::File(file)) => json!({
			"type": "file",
			"path": file.path,
			"tags": file.tags,
			"display_tags": file.display_tags,
		}),
		Some(SearchSelection::Attribute(attribute)) => json!({
			"type": "attribute",
			"name": attribute.name,
			"count": attribute.count,
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

/// Print the JSON representation of the search outcome.
pub(crate) fn print_json(outcome: &SearchOutcome) -> Result<()> {
	println!("{}", format_outcome_json(outcome)?);
	Ok(())
}

#[cfg(test)]
mod tests {
	use frz::{AttributeRow, FileRow};
	use serde_json::Value;

	use super::*;

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
	fn json_format_includes_attribute_selection() {
		let outcome = SearchOutcome {
			accepted: true,
			query: "test".into(),
			selection: Some(SearchSelection::Attribute(AttributeRow::new(
				"attribute",
				3,
			))),
		};

		let json = format_outcome_json(&outcome).expect("json");
		let value: Value = serde_json::from_str(&json).expect("parse");
		assert_eq!(value["selection"]["type"], "attribute");
		assert_eq!(value["selection"]["name"], "attribute");
		assert_eq!(value["selection"]["count"], 3);
	}
}
