//! Configuration — two sources that do not merge:
//! - [`env`] — runtime knobs (test backends, log forcing) via [`ZervRuntimeConfig`].
//! - [`file`] — a repo's committed version policy (`zerv.toml`) via [`ZervFileConfig`].
//!
//! Env controls how the tool runs; file controls what version it outputs.
//!
//! [`ZervRuntimeConfig`]: env::ZervRuntimeConfig
//! [`ZervFileConfig`]: file::ZervFileConfig

pub mod env;
pub mod file;
pub mod merge;

pub use env::*;
pub use file::*;
pub use merge::*;
