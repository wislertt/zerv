use std::env;

#[derive(Debug, Clone, Default)]
pub struct ZervConfig {
    pub test_native_git: bool,
    pub test_docker: bool,
}

impl ZervConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let test_native_git = Self::parse_bool_env("ZERV_TEST_NATIVE_GIT", false)?;
        let test_docker = Self::parse_bool_env("ZERV_TEST_DOCKER", true)?;

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
    use super::*;
    use serial_test::serial;
    use std::env;

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
        let _guard = EnvGuard::new(&["ZERV_TEST_NATIVE_GIT", "ZERV_TEST_DOCKER"]);
        unsafe {
            env::remove_var("ZERV_TEST_NATIVE_GIT");
            env::remove_var("ZERV_TEST_DOCKER");
        }

        let config = ZervConfig::load().expect("Failed to load default config");
        assert!(!config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_native_git_env_var() {
        let _guard = EnvGuard::new(&["ZERV_TEST_NATIVE_GIT", "ZERV_TEST_DOCKER"]);
        unsafe {
            env::remove_var("ZERV_TEST_DOCKER");
            env::set_var("ZERV_TEST_NATIVE_GIT", "true");
        }

        let config = ZervConfig::load().expect("Failed to load config with native git enabled");
        assert!(config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_docker_tests_env_var() {
        let _guard = EnvGuard::new(&["ZERV_TEST_NATIVE_GIT", "ZERV_TEST_DOCKER"]);
        unsafe {
            env::remove_var("ZERV_TEST_NATIVE_GIT");
            env::set_var("ZERV_TEST_DOCKER", "true");
        }

        let config = ZervConfig::load().expect("Failed to load config with docker tests enabled");
        assert!(!config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_both_env_vars() {
        let _guard = EnvGuard::new(&["ZERV_TEST_NATIVE_GIT", "ZERV_TEST_DOCKER"]);
        unsafe {
            env::set_var("ZERV_TEST_NATIVE_GIT", "true");
            env::set_var("ZERV_TEST_DOCKER", "true");
        }

        let config = ZervConfig::load().expect("Failed to load config with both env vars set");
        assert!(config.should_use_native_git());
        assert!(config.should_run_docker_tests());
    }

    #[test]
    #[serial]
    fn test_false_values() {
        let _guard = EnvGuard::new(&["ZERV_TEST_NATIVE_GIT", "ZERV_TEST_DOCKER"]);
        unsafe {
            env::set_var("ZERV_TEST_NATIVE_GIT", "false");
            env::set_var("ZERV_TEST_DOCKER", "false");
        }

        let config = ZervConfig::load().expect("Failed to load config with false values");
        assert!(!config.should_use_native_git());
        assert!(!config.should_run_docker_tests());
    }
}
