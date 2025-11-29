//! Filesystem-specific helpers for building the search experience.
//!
//! This module re-exports the indexer and search pipeline functionality under
//! a single namespace so consumers can reason about filesystem state in one
//! place.

pub mod indexer;
pub mod search;
