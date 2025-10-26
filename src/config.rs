use std::env;

/// Centralized environment variable names used throughout Zerv.
/// Following uv's pattern for maintainability and documentation.
pub struct EnvVars;

impl EnvVars {
    /// Control logging verbosity (standard Rust ecosystem convention).
    ///
    /// Examples:
    /// * `RUST_LOG=zerv=debug` - Debug logs for Zerv
    /// * `RUST_LOG=trace` - Trace-level logging
    /// * `RUST_LOG=zerv::vcs=debug` - Module-specific logging
    ///
    /// See [tracing documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
    pub const RUST_LOG: &'static str = "RUST_LOG";

    /// Use native Git instead of Docker Git for tests (default: false).
    ///
    /// Set to `true` or `1` to enable native Git in test environments.
    pub const ZERV_TEST_NATIVE_GIT: &'static str = "ZERV_TEST_NATIVE_GIT";

    /// Enable Docker-dependent tests (default: true).
    ///
    /// Set to `false` or `0` to skip Docker tests on systems without Docker.
    pub const ZERV_TEST_DOCKER: &'static str = "ZERV_TEST_DOCKER";

    /// Preferred pager program for displaying manual pages.
    ///
    /// Examples:
    /// * `PAGER=less` - Use less pager
    /// * `PAGER=more` - Use more pager
    /// * `PAGER=most` - Use most pager
    ///
    /// If not set, Zerv will fall back to searching for common pagers (less, more, most).
    pub const PAGER: &'static str = "PAGER";
}

#[derive(Debug, Clone, Default)]
pub struct ZervConfig {
    pub test_native_git: bool,
    pub test_docker: bool,
}

impl ZervConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let test_native_git = Self::parse_bool_env(EnvVars::ZERV_TEST_NATIVE_GIT, false)?;
        let test_docker = Self::parse_bool_env(EnvVars::ZERV_TEST_DOCKER, true)?;

        Ok(ZervConfig {
            test_native_git,
            test_docker,
        })
    }

    fn parse_bool_env(var_name: &str, default: bool) -> Result<bool, Box<dyn std::error::Error>> {
        match env::var(var_name) {
            Ok(val) => Ok(val == "true" || val == "1"),
            Err(_) => Ok(default),
        }
    }

    pub fn should_use_native_git(&self) -> bool {
        self.test_native_git
    }

    pub fn should_run_docker_tests(&self) -> bool {
        self.test_docker
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use serial_test::serial;

    use super::*;

    struct EnvGuard {
        vars: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn new(var_names: &[&str]) -> Self {
            let vars = var_names
                .iter()
                .map(|&name| (name.to_string(), env::var(name).ok()))
                .collect();
            Self { vars }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in &self.vars {
                unsafe {
                    match value {
                        Some(val) => env::set_var(name, val),
                        None => env::remove_var(name),
                    }
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_default_config() {
        let _guard = EnvGuard::new(&[EnvVars::ZERV_TEST_NATIVE_GIT, EnvVars::ZERV_TEST_DOCKER]);
        unsafe {
            env::remove_var(EnvVars::ZERV_TEST_NATIVE_GIT);
            env::remove_var(EnvVars::ZERV_TEST_DOCKER);
        }

        let config = ZervConfig::load().expect("Failed to load default config");
        assert!(!config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_native_git_env_var() {
        let _guard = EnvGuard::new(&[EnvVars::ZERV_TEST_NATIVE_GIT, EnvVars::ZERV_TEST_DOCKER]);
        unsafe {
            env::remove_var(EnvVars::ZERV_TEST_DOCKER);
            env::set_var(EnvVars::ZERV_TEST_NATIVE_GIT, "true");
        }

        let config = ZervConfig::load().expect("Failed to load config with native git enabled");
        assert!(config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_docker_tests_env_var() {
        let _guard = EnvGuard::new(&[EnvVars::ZERV_TEST_NATIVE_GIT, EnvVars::ZERV_TEST_DOCKER]);
        unsafe {
            env::remove_var(EnvVars::ZERV_TEST_NATIVE_GIT);
            env::set_var(EnvVars::ZERV_TEST_DOCKER, "true");
        }

        let config = ZervConfig::load().expect("Failed to load config with docker tests enabled");
        assert!(!config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_both_env_vars() {
        let _guard = EnvGuard::new(&[EnvVars::ZERV_TEST_NATIVE_GIT, EnvVars::ZERV_TEST_DOCKER]);
        unsafe {
            env::set_var(EnvVars::ZERV_TEST_NATIVE_GIT, "true");
            env::set_var(EnvVars::ZERV_TEST_DOCKER, "true");
        }

        let config = ZervConfig::load().expect("Failed to load config with both env vars set");
        assert!(config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_false_values() {
        let _guard = EnvGuard::new(&[EnvVars::ZERV_TEST_NATIVE_GIT, EnvVars::ZERV_TEST_DOCKER]);
        unsafe {
            env::set_var(EnvVars::ZERV_TEST_NATIVE_GIT, "false");
            env::set_var(EnvVars::ZERV_TEST_DOCKER, "false");
        }

        let config = ZervConfig::load().expect("Failed to load config with false values");
        assert!(!config.should_use_native_git());
        assert!(!config.should_run_docker_tests());
    }
}
