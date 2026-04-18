//! Shared types and utilities — NO business logic.
//!
//! This module is the ONLY crate-internal dependency allowed by all
//! other modules. It must NEVER import from any other module in the
//! project.
//!
//! **Open for extension:** Any engineer may add new files here for
//! shared types that multiple modules need.

pub mod error;
pub mod schema;
pub mod shard_key;
pub mod types;
