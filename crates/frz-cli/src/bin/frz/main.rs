//! Command-line entry point for the frz file finder application.

mod cli;
mod config;
mod workflow;

use anyhow::Result;
use cli::{OutputFormat, parse_cli, print_json, print_plain};
use config::Config;
use frz::features::tui_app::style;
use workflow::SearchWorkflow;

/// Entry point for the frz command-line application.
fn main() -> Result<()> {
	let cli = parse_cli();

	if cli.list_themes {
		for name in style::names() {
			println!("{name}");
		}
		return Ok(());
	}

	let config = Config::from_cli(&cli)?;

	if cli.print_config {
		println!("Root: {}", config.root.display());
		println!("Threads: {:?}", config.filesystem.threads);
		println!("Max depth: {:?}", config.filesystem.max_depth);
		println!("Hidden files: {}", config.filesystem.include_hidden);
		println!("Follow symlinks: {}", config.filesystem.follow_symlinks);
		println!("Theme: {:?}", config.theme);
	}

	run_search(cli.output, config)
}

/// Execute the search workflow and print output in the chosen format.
fn run_search(format: OutputFormat, config: Config) -> Result<()> {
	let workflow = SearchWorkflow::from_config(config)?;
	let outcome = workflow.run()?;

	match format {
		OutputFormat::Plain => print_plain(&outcome),
		OutputFormat::Json => print_json(&outcome)?,
	}

	Ok(())
}
