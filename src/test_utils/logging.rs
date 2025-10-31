//! Test logging utilities and macros
//!
//! Provides custom logging macros for test code that automatically use the "zerv_test" target,
//! allowing clean separation between test logs and source code logs.
//!
//! # Usage
//!
//! ```rust
//! use zerv::test_utils::logging::{test_debug, test_info, test_error, test_warn};
//!
//! fn test_function() {
//!     test_debug!("Starting test setup");
//!     test_info!("Test result: {}", result);
//!     test_error!("Test failed: {}", error);
//!     test_warn!("Test warning: slow operation");
//! }
//! ```
//!
//! # Filtering
//!
//! Use RUST_LOG to filter test logs separately from source code:
//!
//! ```bash
//! # Show only test logs
//! RUST_LOG=zerv_test=info cargo test
//!
//! # Show all logs (test + source)
//! RUST_LOG=debug cargo test
//!
//! # Show test logs + specific source modules
//! RUST_LOG=zerv_test=info,zerv::cli::flow=debug cargo test
//! ```

/// Custom debug macro for test code with automatic "zerv_test" target
#[macro_export]
macro_rules! test_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "zerv_test", $($arg)*);
    };
}

/// Custom info macro for test code with automatic "zerv_test" target
#[macro_export]
macro_rules! test_info {
    ($($arg:tt)*) => {
        tracing::info!(target: "zerv_test", $($arg)*);
    };
}

/// Custom error macro for test code with automatic "zerv_test" target
#[macro_export]
macro_rules! test_error {
    ($($arg:tt)*) => {
        tracing::error!(target: "zerv_test", $($arg)*);
    };
}

/// Custom warn macro for test code with automatic "zerv_test" target
#[macro_export]
macro_rules! test_warn {
    ($($arg:tt)*) => {
        tracing::warn!(target: "zerv_test", $($arg)*);
    };
}

/// Re-export macros for easier use
pub use crate::{
    test_debug,
    test_error,
    test_info,
    test_warn,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_macros() {
        // These should compile and not panic
        test_debug!("Test debug message: {}", 42);
        test_info!("Test info message: {}", "hello");
        test_error!("Test error message: {}", "error");
        test_warn!("Test warn message: {}", "warning");
    }
}
