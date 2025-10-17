use clap::Arg;

mod format;
mod render;
mod style;

use format::style_base_help;
use render::{
    render_default_value_annotation, render_env_annotation, render_possible_values_annotation,
};
use style::append_muted_annotation;

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
