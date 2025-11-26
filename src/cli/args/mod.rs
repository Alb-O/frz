mod definitions;
mod options;
mod styles;

use clap::Parser;
pub(crate) use definitions::CliArgs;
pub(crate) use options::OutputFormat;

/// Parse command line arguments into the strongly typed [`CliArgs`] structure.
pub(crate) fn parse_cli() -> CliArgs {
	CliArgs::parse()
}
