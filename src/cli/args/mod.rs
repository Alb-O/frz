mod command;
mod definitions;
mod options;
mod styles;

pub(crate) use command::parse_cli;
pub(crate) use definitions::CliArgs;
pub(crate) use options::OutputFormat;

#[cfg(test)]
mod tests;
