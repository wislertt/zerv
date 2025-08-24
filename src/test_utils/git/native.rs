use super::{GitOperations, TestDir};
use std::io;
use std::process::Command;

/// Native Git implementation for CI testing
#[derive(Default)]
pub struct NativeGit;

impl NativeGit {
    pub fn new() -> Self {
        Self
    }
}

impl GitOperations for NativeGit {
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(test_dir.path())
            .output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
