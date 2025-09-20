use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use zerv::test_utils::TestOutput;

/// Test command utility for running zerv CLI with assertions
pub struct TestCommand {
    cmd: Command,
    #[allow(dead_code)]
    current_dir: Option<PathBuf>,
}

impl Default for TestCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCommand {
    /// Create a new test command for zerv binary
    pub fn new() -> Self {
        // Try to use built binary first, fallback to cargo run
        let binary_path = std::env::current_dir().unwrap().join("target/debug/zerv");

        let cmd = if binary_path.exists() {
            Command::new(binary_path)
        } else {
            let mut cmd = Command::new("cargo");
            cmd.args(["run", "--bin", "zerv", "--"]);
            cmd
        };

        Self {
            cmd,
            current_dir: None,
        }
    }

    /// Add an argument to the command
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    /// Add multiple arguments to the command
    #[allow(dead_code)]
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd.args(args);
        self
    }

    /// Set the current directory for the command
    #[allow(dead_code)]
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.current_dir = Some(dir.as_ref().to_path_buf());
        self.cmd.current_dir(&dir);
        self
    }

    /// Execute the command and return output
    pub fn output(&mut self) -> io::Result<Output> {
        self.cmd.output()
    }

    /// Execute and assert success
    pub fn assert_success(&mut self) -> TestOutput {
        let output = self.output().expect("Failed to execute command");
        assert!(
            output.status.success(),
            "Command failed with exit code: {:?}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        TestOutput::new(output)
    }

    /// Execute and assert failure
    #[allow(dead_code)]
    pub fn assert_failure(&mut self) -> TestOutput {
        let output = self.output().expect("Failed to execute command");
        assert!(
            !output.status.success(),
            "Expected command to fail but it succeeded"
        );
        TestOutput::new(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::process::Command;

    #[test]
    fn test_command_new() {
        let cmd = TestCommand::new();
        assert!(cmd.current_dir.is_none());
    }

    #[rstest]
    #[case("--version")]
    #[case("--help")]
    #[case("-V")]
    #[case("-h")]
    fn test_command_arg_variations(#[case] arg: &str) {
        let mut cmd = TestCommand::new();
        cmd.arg(arg);
    }

    #[test]
    fn test_command_args() {
        let mut cmd = TestCommand::new();
        cmd.args(["--version", "--help"]);
    }

    #[test]
    fn test_command_current_dir() {
        let mut cmd = TestCommand::new();
        let temp_dir = std::env::temp_dir();
        cmd.current_dir(&temp_dir);
        assert_eq!(cmd.current_dir, Some(temp_dir));
    }

    #[test]
    fn test_command_assert_failure() {
        let mut cmd = Command::new("false");
        let output = cmd.output().unwrap();
        assert!(!output.status.success());
    }

    #[test]
    fn test_test_command_assert_failure() {
        // Create a TestCommand that will fail by using an invalid argument
        let mut cmd = TestCommand::new();
        cmd.arg("--invalid-flag-that-does-not-exist");
        let _test_output = cmd.assert_failure();
        // If we reach here, assert_failure worked correctly
    }
}
