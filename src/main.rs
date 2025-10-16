#[cfg(feature = "fs")]
mod cli;
#[cfg(feature = "fs")]
mod settings;
#[cfg(feature = "fs")]
mod workflow;

#[cfg(feature = "fs")]
use anyhow::Result;
#[cfg(feature = "fs")]
use cli::{OutputFormat, parse_cli, print_json, print_plain};
#[cfg(feature = "fs")]
use settings::ResolvedConfig;
#[cfg(feature = "fs")]
use workflow::SearchWorkflow;

#[cfg(feature = "fs")]
fn main() -> Result<()> {
    let cli = parse_cli();

    if cli.list_themes {
        for name in frz_tui::theme::NAMES {
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
    let workflow = SearchWorkflow::from_config(settings)?;
    let outcome = workflow.run()?;

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
