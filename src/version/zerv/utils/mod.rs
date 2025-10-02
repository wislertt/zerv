pub mod general;
pub mod timestamp;

// Re-export the main functions for backward compatibility
pub use general::{extract_core_values, normalize_pre_release_label};
pub use timestamp::resolve_timestamp;
