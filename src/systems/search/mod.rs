mod commands;
pub mod extension;
mod worker;

pub(crate) use commands::{SearchCommand, SearchResult};
pub use extension::config_for_query;
pub(crate) use worker::spawn;
