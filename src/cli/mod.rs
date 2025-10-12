#[cfg(feature = "fs")]
mod annotations;
#[cfg(feature = "fs")]
mod args;
#[cfg(feature = "fs")]
mod output;

#[cfg(feature = "fs")]
pub(crate) use args::{CliArgs, OutputFormat, parse_cli};
#[cfg(feature = "fs")]
pub(crate) use output::{print_json, print_plain};
