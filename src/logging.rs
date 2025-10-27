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
    let rust_log_env = std::env::var(EnvVars::RUST_LOG);

    // ZERV_LOG_DEBUG: Unique keyword for debugging logging initialization in Ubuntu CI
    // Check OS to help debug Ubuntu-specific issues
    let os_info = if cfg!(target_os = "linux") {
        "LINUX"
    } else if cfg!(target_os = "macos") {
        "MACOS"
    } else if cfg!(target_os = "windows") {
        "WINDOWS"
    } else {
        "UNKNOWN"
    };

    eprintln!(
        "ZERV_LOG_DEBUG: [{}] init_logging called with verbose={}, RUST_LOG={:?}",
        os_info, verbose, rust_log_env
    );

    let filter = if let Ok(rust_log) = rust_log_env {
        eprintln!(
            "ZERV_LOG_DEBUG: Using RUST_LOG environment variable: {}",
            rust_log
        );

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
        eprintln!("ZERV_LOG_DEBUG: Using verbose mode, setting zerv=debug");
        EnvFilter::new("zerv=debug")
    } else {
        eprintln!("ZERV_LOG_DEBUG: Using default error level logging");
        EnvFilter::new("error")
    };

    eprintln!("ZERV_LOG_DEBUG: Final filter applied");
    let _result = fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .try_init();

    eprintln!(
        "ZERV_LOG_DEBUG: Logging initialization completed with result: {:?}",
        _result
    );
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
