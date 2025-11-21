use clap::Parser;

use super::definitions::CliArgs;

/// Parse command line arguments into the strongly typed [`CliArgs`] structure.
pub(crate) fn parse_cli() -> CliArgs {
	CliArgs::parse()
}
