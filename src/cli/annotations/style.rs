use clap::builder::{
    StyledStr,
    styling::{AnsiColor, Color, Style},
};

/// Return the muted style used to annotate clap help metadata.
pub(super) fn muted_default_style() -> Style {
    Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
        .dimmed()
}

/// Append an annotation using the muted help style.
pub(super) fn append_muted_annotation(target: &mut StyledStr, annotation: &str) {
    let style = muted_default_style();
    let _ = std::fmt::write(target, format_args!("{style}{annotation}{style:#}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_muted_annotation_adds_text() {
        let mut text = StyledStr::new();
        append_muted_annotation(&mut text, "[note]");
        assert_eq!(text.to_string(), "[note]");
    }
}
