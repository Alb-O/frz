use clap::Arg;
use clap::builder::{
    StyledStr,
    styling::{AnsiColor, Color, Style},
};

/// Apply dimmed styling to relevant clap annotations for improved readability.
pub(crate) fn dim_cli_annotations(mut arg: Arg) -> Arg {
    let help_text = arg
        .get_help()
        .cloned()
        .map(|help| help.to_string())
        .unwrap_or_default();
    let mut styled_help = style_base_help(&help_text);
    let mut has_help = !help_text.is_empty();
    let mentions_default = help_text.contains("(default:");

    if let Some(annotation) = render_possible_values_annotation(&arg) {
        arg = arg.hide_possible_values(true);
        if has_help {
            styled_help.push_str(" ");
        }
        append_muted_annotation(&mut styled_help, &annotation);
        has_help = true;
    }

    if !mentions_default && let Some(annotation) = render_default_value_annotation(&arg) {
        arg = arg.hide_default_value(true);
        if has_help {
            styled_help.push_str(" ");
        }
        append_muted_annotation(&mut styled_help, &annotation);
        has_help = true;
    }

    if let Some(annotation) = render_env_annotation(&arg) {
        arg = arg.hide_env(true);
        if has_help {
            styled_help.push_str(" ");
        }
        append_muted_annotation(&mut styled_help, &annotation);
        has_help = true;
    }

    if has_help {
        arg = arg.help(styled_help);
    }

    arg
}

/// Highlight default annotations within clap-generated help text.
pub(crate) fn highlight_help_annotations(text: &str) -> Option<StyledStr> {
    const ANNOTATIONS: &[(&str, char)] = &[("(default: ", ')')];

    let mut styled = StyledStr::new();
    let mut last = 0;
    let mut changed = false;
    let mut cursor = 0;

    while cursor < text.len() {
        let mut best_match: Option<(usize, usize)> = None;

        for &(pattern, terminator) in ANNOTATIONS {
            if let Some(rel_start) = text[cursor..].find(pattern) {
                let start = cursor + rel_start;
                if let Some(rel_end) = text[start..].find(terminator) {
                    let end = start + rel_end + 1;
                    if best_match.is_none_or(|(current_start, _)| start < current_start) {
                        best_match = Some((start, end));
                    }
                }
            }
        }

        let (start, end) = match best_match {
            Some(bounds) => bounds,
            None => break,
        };

        styled.push_str(&text[last..start]);

        let style = muted_default_style();
        std::fmt::write(
            &mut styled,
            format_args!("{style}{}{style:#}", &text[start..end]),
        )
        .ok()?;

        last = end;
        cursor = end;
        changed = true;
    }

    if !changed {
        return None;
    }

    styled.push_str(&text[last..]);
    Some(styled)
}

/// Return the muted style used to annotate clap help metadata.
fn muted_default_style() -> Style {
    Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
        .dimmed()
}

/// Create a baseline styled help entry for clap arguments.
pub(crate) fn style_base_help(text: &str) -> StyledStr {
    if text.is_empty() {
        return StyledStr::new();
    }

    highlight_help_annotations(text).unwrap_or_else(|| {
        let mut styled = StyledStr::new();
        styled.push_str(text);
        styled
    })
}

/// Append an annotation using the muted help style.
fn append_muted_annotation(target: &mut StyledStr, annotation: &str) {
    let style = muted_default_style();
    let _ = std::fmt::write(target, format_args!("{style}{annotation}{style:#}"));
}

/// Render clap possible value annotations for display.
pub(crate) fn render_possible_values_annotation(arg: &Arg) -> Option<String> {
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
pub(crate) fn render_default_value_annotation(arg: &Arg) -> Option<String> {
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
pub(crate) fn render_env_annotation(arg: &Arg) -> Option<String> {
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

    #[test]
    fn highlight_detects_defaults() {
        let text = "Use the option (default: value) to toggle";
        let styled = highlight_help_annotations(text).expect("highlighted");
        assert_eq!(styled.to_string(), text);
    }
}
