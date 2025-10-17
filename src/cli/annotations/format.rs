use clap::builder::StyledStr;

use super::style::muted_default_style;

/// Highlight default annotations within clap-generated help text.
pub(super) fn highlight_help_annotations(text: &str) -> Option<StyledStr> {
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

/// Create a baseline styled help entry for clap arguments.
pub(super) fn style_base_help(text: &str) -> StyledStr {
    if text.is_empty() {
        return StyledStr::new();
    }

    highlight_help_annotations(text).unwrap_or_else(|| {
        let mut styled = StyledStr::new();
        styled.push_str(text);
        styled
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_detects_defaults() {
        let text = "Use the option (default: value) to toggle";
        let styled = highlight_help_annotations(text).expect("highlighted");
        assert_eq!(styled.to_string(), text);
    }
}
