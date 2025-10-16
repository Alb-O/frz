mod annotations;
mod args;
mod output;

pub(crate) use args::{CliArgs, OutputFormat, parse_cli};
pub(crate) use output::{print_json, print_plain};
