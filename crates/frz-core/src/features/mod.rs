//! Feature modules containing the main vertical slices of functionality.
//!
//! Each feature module encapsulates a complete slice of the application:
//! - [`search_pipeline`]: Fuzzy matching, scoring, and streaming infrastructure
//! - [`filesystem_indexer`]: Filesystem traversal, caching, and file discovery

pub mod filesystem_indexer;
pub mod search_pipeline;
