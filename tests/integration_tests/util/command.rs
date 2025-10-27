use std::ffi::OsStr;
use std::io::{
    self,
    Write,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Output,
    Stdio,
};

use zerv::test_utils::TestOutput;

/// Test command utility for running zerv CLI with assertions
pub struct TestCommand {
    cmd: Command,
    #[allow(dead_code)]
    current_dir: Option<PathBuf>,
    stdin_input: Option<String>,
}

impl Default for TestCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCommand {
    /// Create a new test command for zerv binary
    pub fn new() -> Self {
        // Get the workspace root directory
        let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Fallback: find workspace root by looking for Cargo.toml
                let mut current = std::env::current_dir().unwrap();
                while !current.join("Cargo.toml").exists() {
                    if let Some(parent) = current.parent() {
                        current = parent.to_path_buf();
                    } else {
                        // If we can't find Cargo.toml, use current dir
                        return std::env::current_dir().unwrap();
                    }
                }
                current
            });

        // Try multiple binary locations
        let binary_paths = [
            workspace_root.join("target/debug/zerv"),
            workspace_root.join("target/debug/zerv.exe"), // Windows
            workspace_root.join("target/release/zerv"),
            workspace_root.join("target/release/zerv.exe"), // Windows
        ];

        let binary_path = binary_paths.iter().find(|path| path.exists());

        let mut cmd = if let Some(path) = binary_path {
            Command::new(path)
        } else {
            // Fallback to cargo run from workspace root
            let mut cmd = Command::new("cargo");
            cmd.args(["run", "--bin", "zerv", "--"]);
            cmd.current_dir(&workspace_root);
            cmd
        };

        // Ensure we don't inherit the current directory for cargo run
        // This prevents cargo from looking for Cargo.toml in test directories
        if binary_path.is_none() {
            cmd.current_dir(&workspace_root);
        }

        Self {
            cmd,
            current_dir: None,
            stdin_input: None,
        }
    }

    /// Add an argument to the command
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    /// Add multiple arguments to the command
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd.args(args);
        self
    }

    /// Add arguments from a shell-like string
    ///
    /// Uses POSIX shell word splitting (via `shlex` crate), which means it behaves
    /// exactly like your terminal shell when splitting arguments.
    ///
    /// Supports:
    /// - Single quotes: 'arg with spaces' (preserves everything literally)
    /// - Double quotes: "arg with spaces" (allows escape sequences)
    /// - Backslash escapes: \' \" \\ \n \t \r
    /// - Mixed quoting: --flag="value with 'quotes'"
    /// - Flag forms: --source stdin and --source=stdin (both work)
    ///
    /// Examples:
    /// ```
    /// .args_from_str("version --source stdin --output-format semver")
    /// .args_from_str("version --template 'v{{major}}.{{minor}}'")
    /// .args_from_str(r#"version --template "version {{major}}.{{minor}}""#)
    /// .args_from_str(r"version --prefix 'v' --suffix '-dev'")
    /// ```
    pub fn args_from_str<S: AsRef<str>>(&mut self, args_str: S) -> &mut Self {
        if let Some(args) = shlex::split(args_str.as_ref()) {
            self.cmd.args(args);
        } else {
            // If shlex fails to parse (e.g., unclosed quote), pass the string as-is
            // This will likely fail when the command runs, which is the desired behavior
            self.cmd.arg(args_str.as_ref());
        }
        self
    }

    /// Set the current directory for the command
    #[allow(dead_code)]
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.current_dir = Some(dir.as_ref().to_path_buf());
        self.cmd.current_dir(&dir);
        self
    }

    /// Set stdin input for the command
    #[allow(dead_code)]
    pub fn stdin<S: Into<String>>(&mut self, input: S) -> &mut Self {
        self.stdin_input = Some(input.into());
        self
    }

    /// Set an environment variable for the command
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.cmd.env(key, val);
        self
    }

    /// Execute the command and return output
    pub fn output(&mut self) -> io::Result<Output> {
        if let Some(ref input) = self.stdin_input {
            // Need to use piped stdin
            self.cmd
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let mut child = self.cmd.spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                let result = stdin.write_all(input.as_bytes());
                drop(stdin);

                if let Err(e) = result {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                    } else {
                        return Err(e);
                    }
                }
            }

            child.wait_with_output()
        } else {
            self.cmd.output()
        }
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

    /// Convenience method: create command, provide stdin, and return stdout
    pub fn run_with_stdin(args: &str, input: String) -> String {
        Self::new()
            .args_from_str(args)
            .stdin(input)
            .assert_success()
            .stdout()
            .trim()
            .to_string()
    }

    /// Convenience method: create command, provide stdin, and expect failure
    pub fn run_with_stdin_expect_fail(args: &str, input: String) -> String {
        Self::new()
            .args_from_str(args)
            .stdin(input)
            .assert_failure()
            .stderr()
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_command_new() {
        let cmd = TestCommand::new();
        assert!(cmd.current_dir.is_none());
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
        let mut cmd = TestCommand::new();
        cmd.arg("--invalid-flag-that-does-not-exist");
        let _test_output = cmd.assert_failure();
    }

    #[rstest]
    #[case("--version", "zerv")]
    #[case("--help", "Usage")]
    #[case("-V", "zerv")]
    #[case("-h", "Usage")]
    fn test_args_from_str_basic_flags(#[case] args: &str, #[case] expected_output: &str) {
        let mut cmd = TestCommand::new();
        cmd.args_from_str(args);
        let output = cmd.assert_success();
        assert!(output.stdout().contains(expected_output));
    }

    #[rstest]
    #[case(
        None,
        r#"version --source stdin --output-template "v{{major}}.{{minor}}""#,
        "v1.2"
    )]
    #[case(
        None,
        r#"version --source stdin --output-template "{{major}}.{{minor}}.{{patch}}""#,
        "1.2.3"
    )]
    #[case(Some(2), r#"version --source stdin --output-template "{{#if epoch}}{{epoch}}!{{/if}}{{major}}.{{minor}}.{{patch}}""#, "2!1.2.3")]
    #[case(
        None,
        r#"version --source stdin --output-template "Version {{major}}.{{minor}}""#,
        "Version 1.2"
    )]
    #[case(
        None,
        r#"version --source=stdin --output-template="v{{major}}.{{minor}}""#,
        "v1.2"
    )]
    fn test_args_from_str_with_templates(
        #[case] epoch: Option<u64>,
        #[case] args: &str,
        #[case] expected: &str,
    ) {
        use zerv::test_utils::ZervFixture;

        let mut fixture = ZervFixture::new().with_version(1, 2, 3);
        if let Some(e) = epoch {
            fixture = fixture.with_epoch(e);
        }
        let zerv_ron = fixture.build().to_string();

        let mut cmd = TestCommand::new();
        cmd.args_from_str(args).stdin(zerv_ron);
        let output = cmd.assert_success();
        assert_eq!(output.stdout().trim(), expected);
    }

    #[rstest]
    #[case("version --source stdin", "1.2.3")]
    #[case("version --source stdin --output-format semver", "1.2.3")]
    #[case("version --source stdin --output-format pep440", "1.2.3")]
    #[case(
        r#"version --source stdin --output-template "v{{major}}.{{minor}}""#,
        "v1.2"
    )]
    fn test_run_with_stdin(#[case] args: &str, #[case] expected: &str) {
        use zerv::test_utils::ZervFixture;

        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();
        let output = TestCommand::run_with_stdin(args, zerv_ron);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_run_with_stdin_expect_fail() {
        use zerv::test_utils::ZervFixture;

        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();
        let result = TestCommand::run_with_stdin_expect_fail(
            "version --source stdin --invalid-flag",
            zerv_ron,
        );

        // Verify it returns stderr directly
        assert!(result.contains("unexpected argument") || result.contains("invalid"));
    }
}
