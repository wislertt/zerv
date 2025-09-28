pub mod core;
mod display;
mod parser;
pub mod schema; // New module
#[cfg(test)]
mod test_schema_assertions;
pub mod utils;
pub mod vars; // New module

// Core types
pub use core::{PreReleaseLabel, PreReleaseVar, Zerv};
// Vars types
pub use vars::ZervVars;
// Schema types
pub use schema::{Component, ZervSchema};
// Utilities
pub use utils::{normalize_pre_release_label, resolve_timestamp};
