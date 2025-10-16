mod commands;
pub mod plugin;
mod worker;

pub(crate) use commands::{SearchCommand, SearchResult};
pub(crate) use worker::spawn;

pub use plugin::config_for_query;
