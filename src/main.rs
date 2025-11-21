mod cli;
mod settings;
mod workflow;

use anyhow::Result;
use cli::{OutputFormat, parse_cli, print_json, print_plain};
use settings::ResolvedConfig;
use workflow::SearchWorkflow;

fn main() -> Result<()> {
	let cli = parse_cli();

	if cli.list_themes {
		for name in frz::tui::theme::names() {
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
