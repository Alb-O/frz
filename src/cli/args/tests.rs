use clap::{CommandFactory, FromArgMatches};

use super::command::tinted_cli_command;
use super::{CliArgs, OutputFormat};

#[test]
fn command_supports_custom_styles() {
	let command = tinted_cli_command();
	assert!(command.get_about().is_some());
}

#[test]
fn parse_cli_accepts_default_arguments() {
	let command = CliArgs::command();
	let mut matches = command.get_matches_from(vec!["frz"]);
	let parsed = CliArgs::from_arg_matches_mut(&mut matches).expect("parses");
	assert_eq!(parsed.output, OutputFormat::Plain);
}
