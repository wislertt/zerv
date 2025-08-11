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
    use std::process::Command;

    #[test]
    fn test_output_methods() {
        let output = Command::new("/bin/echo")
            .arg("test output")
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        assert!(test_output.stdout().contains("test output"));
    }

    #[test]
    fn test_output_stderr() {
        let output = Command::new("/bin/sh")
            .args(["-c", "echo 'error output' >&2"])
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        assert!(test_output.stderr().contains("error output"));
    }

    #[test]
    fn test_output_assert_stdout_contains() {
        let output = Command::new("/bin/echo")
            .arg("hello world test")
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        test_output.assert_stdout_contains("world");
    }

    #[test]
    fn test_output_assert_stderr_contains() {
        let output = Command::new("/bin/sh")
            .args(["-c", "echo 'error: something failed' >&2"])
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        test_output.assert_stderr_contains("failed");
    }

    #[test]
    fn test_output_assert_stdout_eq() {
        let output = Command::new("/bin/echo")
            .arg("exact match")
            .output()
            .unwrap();
        let test_output = TestOutput::new(output);
        test_output.assert_stdout_eq("exact match");
    }
}
