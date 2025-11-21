use clap::Arg;

/// Render clap possible value annotations for display.
pub(super) fn render_possible_values_annotation(arg: &Arg) -> Option<String> {
	if !arg.get_action().takes_values() {
		return None;
	}

	let values = arg.get_possible_values();
	if values.is_empty() {
		return None;
	}

	let mut visible = Vec::new();
	for value in values {
		if value.is_hide_set() {
			continue;
		}

		let name = value.get_name();
		let formatted = if name.chars().any(char::is_whitespace) {
			format!("{name:?}")
		} else {
			name.to_string()
		};
		visible.push(formatted);
	}

	if visible.is_empty() {
		return None;
	}

	Some(format!("[possible values: {}]", visible.join(", ")))
}

/// Render clap default value annotations with optional quoting.
pub(super) fn render_default_value_annotation(arg: &Arg) -> Option<String> {
	let defaults = arg.get_default_values();
	if defaults.is_empty() {
		return None;
	}

	let mut rendered = Vec::new();
	for value in defaults {
		let text = value.to_string_lossy();
		if text.trim().is_empty() {
			continue;
		}

		let formatted = if text.chars().any(char::is_whitespace) {
			format!("{text:?}")
		} else {
			text.to_string()
		};
		rendered.push(formatted);
	}

	if rendered.is_empty() {
		return None;
	}

	Some(format!("(default: {})", rendered.join(", ")))
}

/// Render environment variable annotations for clap arguments.
pub(super) fn render_env_annotation(arg: &Arg) -> Option<String> {
	if let Some(env) = arg.get_env() {
		let name = env.to_string_lossy();
		if name.trim().is_empty() {
			return None;
		}

		Some(format!("[env: {}=]", name))
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn possible_values_skip_hidden_and_quote_whitespace() {
		let arg = Arg::new("mode")
			.value_parser(["fast", "slow mode"])
			.hide_possible_values(false);

		let annotation = render_possible_values_annotation(&arg).expect("annotation");
		assert_eq!(annotation, "[possible values: fast, \"slow mode\"]");
	}

	#[test]
	fn default_values_ignore_blank_entries() {
		let arg = Arg::new("threads").default_values(["4", " "]);

		let annotation = render_default_value_annotation(&arg).expect("annotation");
		assert_eq!(annotation, "(default: 4)");
	}

	#[test]
	fn env_annotations_trim_names() {
		let arg = Arg::new("config").env("FRZ_CONFIG");
		let annotation = render_env_annotation(&arg).expect("annotation");
		assert_eq!(annotation, "[env: FRZ_CONFIG=]");
	}
}
