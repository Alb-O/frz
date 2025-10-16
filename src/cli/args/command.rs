use clap::{Command, CommandFactory, FromArgMatches};

use crate::cli::annotations::dim_cli_annotations;

use super::definitions::CliArgs;

/// Parse command line arguments into the strongly typed [`CliArgs`] structure.
pub(crate) fn parse_cli() -> CliArgs {
    let mut matches = tinted_cli_command().get_matches();
    CliArgs::from_arg_matches_mut(&mut matches).unwrap_or_else(|err| err.exit())
}

/// Apply styling customisation to the generated clap command.
pub(super) fn tinted_cli_command() -> Command {
    CliArgs::command().mut_args(dim_cli_annotations)
}
