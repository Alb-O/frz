#[cfg(feature = "fs")]
mod cli;
#[cfg(feature = "fs")]
mod settings;

#[cfg(feature = "fs")]
use anyhow::Result;
#[cfg(feature = "fs")]
use cli::{OutputFormat, parse_cli, print_json, print_plain};
#[cfg(feature = "fs")]
use frz::{SearchMode, SearchUi};
#[cfg(feature = "fs")]
use settings::ResolvedConfig;

#[cfg(feature = "fs")]
fn main() -> Result<()> {
    let cli = parse_cli();

    if cli.list_themes {
        for name in frz::theme::NAMES {
            println!("{name}");
        }
        return Ok(());
    }

    let resolved = settings::load(&cli)?;

    if cli.print_config {
        resolved.print_summary();
    }

    run_search(cli.output, resolved)
}

#[cfg(feature = "fs")]
/// Execute the search workflow and print output in the chosen format.
fn run_search(format: OutputFormat, settings: ResolvedConfig) -> Result<()> {
    let ResolvedConfig {
        root,
        filesystem,
        input_title,
        initial_query,
        theme,
        start_mode,
        ui,
        facet_headers,
        file_headers,
    } = settings;

    let mut search_ui = SearchUi::filesystem_with_options(root, filesystem)?;

    if let Some(title) = input_title {
        search_ui = search_ui.with_input_title(title);
    }

    search_ui = search_ui.with_ui_config(ui);
    search_ui = search_ui.with_initial_query(initial_query);

    if let Some(theme) = theme {
        search_ui = search_ui.with_theme_name(&theme);
    }

    if let Some(mode) = start_mode {
        search_ui = search_ui.with_start_mode(mode);
    }

    if let Some(headers) = facet_headers {
        let refs: Vec<&str> = headers.iter().map(|header| header.as_str()).collect();
        search_ui = search_ui.with_headers_for(SearchMode::FACETS, refs);
    }

    if let Some(headers) = file_headers {
        let refs: Vec<&str> = headers.iter().map(|header| header.as_str()).collect();
        search_ui = search_ui.with_headers_for(SearchMode::FILES, refs);
    }

    let outcome = search_ui.run()?;

    match format {
        OutputFormat::Plain => print_plain(&outcome),
        OutputFormat::Json => print_json(&outcome)?,
    }

    Ok(())
}

#[cfg(not(feature = "fs"))]
fn main() {
    eprintln!("The frz binary requires the 'fs' feature to be enabled.");
}
