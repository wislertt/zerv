use std::process::Output;

/// Wrapper for command output with assertion helpers
pub struct TestOutput {
    output: Output,
}

impl TestOutput {
    pub fn new(output: Output) -> Self {
        Self { output }
    }

    /// Get stdout as string
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).to_string()
    }

    /// Get stderr as string
    #[allow(dead_code)]
    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).to_string()
    }

    /// Assert stdout contains text
    pub fn assert_stdout_contains(&self, text: &str) -> &Self {
        let stdout = self.stdout();
        assert!(
            stdout.contains(text),
            "Expected stdout to contain '{text}', but got: {stdout}"
        );
        self
    }

    /// Assert stderr contains text
    #[allow(dead_code)]
    pub fn assert_stderr_contains(&self, text: &str) -> &Self {
        let stderr = self.stderr();
        assert!(
            stderr.contains(text),
            "Expected stderr to contain '{text}', but got: {stderr}"
        );
        self
    }

    /// Assert stdout equals text
    #[allow(dead_code)]
    pub fn assert_stdout_eq(&self, text: &str) -> &Self {
        let stdout = self.stdout();
        let trimmed = stdout.trim();
        assert_eq!(trimmed, text, "Expected stdout to equal '{text}'");
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::process::Command;

    #[rstest]
    #[case("test output", "test", true)]
    #[case("hello world", "world", true)]
    #[case("hello world", "foo", false)]
    fn test_output_stdout_contains(
        #[case] output: &str,
        #[case] search: &str,
        #[case] should_contain: bool,
    ) {
        #[cfg(unix)]
        let cmd_output = Command::new("/bin/echo").arg(output).output().unwrap();
        #[cfg(windows)]
        let cmd_output = Command::new("cmd")
            .args(["/C", "echo", output])
            .output()
            .unwrap();
        let test_output = TestOutput::new(cmd_output);

        if should_contain {
            test_output.assert_stdout_contains(search);
        } else {
            let result = std::panic::catch_unwind(|| {
                test_output.assert_stdout_contains(search);
            });
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_output_stderr() {
        #[cfg(unix)]
        let output = Command::new("/bin/sh")
            .args(["-c", "echo 'error output' >&2"])
            .output()
            .unwrap();
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", "echo error output 1>&2"])
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        assert!(test_output.stderr().contains("error output"));
    }

    #[test]
    fn test_output_assert_stdout_contains() {
        #[cfg(unix)]
        let output = Command::new("/bin/echo")
            .arg("hello world test")
            .output()
            .unwrap();
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", "echo", "hello world test"])
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        test_output.assert_stdout_contains("world");
    }

    #[test]
    fn test_output_assert_stderr_contains() {
        #[cfg(unix)]
        let output = Command::new("/bin/sh")
            .args(["-c", "echo 'error: something failed' >&2"])
            .output()
            .unwrap();
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", "echo error: something failed 1>&2"])
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        test_output.assert_stderr_contains("failed");
    }

    #[test]
    fn test_output_assert_stdout_eq() {
        #[cfg(unix)]
        let output = Command::new("/bin/echo")
            .arg("exact match")
            .output()
            .unwrap();
        #[cfg(windows)]
        let output = Command::new("cmd")
            .args(["/C", "echo", "exact match"])
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        test_output.assert_stdout_eq("exact match");
    }
}
