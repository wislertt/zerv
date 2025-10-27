use tracing_subscriber::{
    EnvFilter,
    fmt,
};

use crate::config::EnvVars;

/// Initialize logging based on --verbose flag and RUST_LOG environment variable
///
/// Verbosity levels (simple and practical):
/// - false (default): error only
/// - true (-v / --verbose): debug (sufficient for all debugging)
///
/// Priority order:
/// 1. RUST_LOG environment variable (if set) - full control
/// 2. --verbose flag - enables debug level
/// 3. Default - error level only (Rust standard)
pub fn init_logging(verbose: bool) {
    let filter = if let Ok(rust_log) = std::env::var(EnvVars::RUST_LOG) {
        // If RUST_LOG is set to off, error, or warn, use it directly
        if rust_log == "off" || rust_log == "error" || rust_log == "warn" {
            EnvFilter::new(rust_log)
        } else {
            // Otherwise, parse it normally but ensure no debug level leaks through
            let mut filter = EnvFilter::new(rust_log);
            // Forcefully disable debug level for all crates, including handlebars
            filter = filter.add_directive("handlebars=error".parse().unwrap());
            filter = filter.add_directive("zerv=error".parse().unwrap());
            filter
        }
    } else if verbose {
        EnvFilter::new("zerv=debug")
    } else {
        EnvFilter::new("error")
    };

    let _result = fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            init_logging(false);
        });
        assert!(
            result.is_ok(),
            "init_logging should not panic with verbose=false"
        );
    }

    #[test]
    fn test_init_logging_with_verbose_flag() {
        let result = std::panic::catch_unwind(|| {
            init_logging(true);
        });
        assert!(
            result.is_ok(),
            "init_logging should not panic with verbose=true"
        );
    }

    #[test]
    fn test_init_logging_with_rust_log_env() {
        unsafe {
            std::env::set_var(EnvVars::RUST_LOG, "debug");
        }
        let result = std::panic::catch_unwind(|| {
            init_logging(false);
        });
        unsafe {
            std::env::remove_var(EnvVars::RUST_LOG);
        }
        assert!(
            result.is_ok(),
            "init_logging should not panic with RUST_LOG set"
        );
    }
}
