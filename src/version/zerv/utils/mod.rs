pub mod general;
pub mod timestamp;

// Re-export the main functions for backward compatibility
pub use general::extract_core_values;
pub use timestamp::resolve_timestamp;
