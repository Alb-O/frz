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
		None => println!("No selection"),
	}
}

/// Format the search outcome as a JSON string.
pub(crate) fn format_outcome_json(outcome: &SearchOutcome) -> Result<String> {
	let selection = match &outcome.selection {
		Some(SearchSelection::File(file)) => json!({
			"type": "file",
			"path": file.path,
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
	use frz::FileRow;
	use serde_json::Value;

	use super::*;

	#[test]
	fn json_format_includes_file_selection() {
		let outcome = SearchOutcome {
			accepted: true,
			query: "test".into(),
			selection: Some(SearchSelection::File(FileRow::new("path"))),
		};

		let json = format_outcome_json(&outcome).expect("json");
		let value: Value = serde_json::from_str(&json).expect("parse");
		assert_eq!(value["selection"]["type"], "file");
		assert_eq!(value["selection"]["path"], "path");
	}
}
