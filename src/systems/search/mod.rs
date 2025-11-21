mod commands;
mod worker;

pub use crate::extensions::api::search::config_for_query;
pub(crate) use commands::{SearchCommand, SearchResult};
pub(crate) use worker::spawn;
