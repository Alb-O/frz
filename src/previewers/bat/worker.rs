use std::path::PathBuf;

use bat::PrettyPrinter;

/// Runs `bat` in the background to produce syntax highlighted output for the
/// preview pane.
pub(super) fn render_file(
    path: PathBuf,
    width: u16,
    bat_theme: Option<&str>,
    git_modifications: bool,
) -> Result<String, String> {
    if width == 0 {
        return Ok(String::new());
    }

    let mut printer = PrettyPrinter::new();
    let term_width = usize::from(width.max(1));
    printer
        .input_file(path.as_path())
        .term_width(term_width)
        .header(false)
        .grid(false)
        .line_numbers(true)
        .vcs_modification_markers(git_modifications)
        .snip(false);

    if let Some(theme) = bat_theme {
        printer.theme(theme);
    }

    let mut output = String::new();
    match printer.print_with_writer(Some(&mut output)) {
        Ok(_) => Ok(output),
        Err(err) => Err(err.to_string()),
    }
}
