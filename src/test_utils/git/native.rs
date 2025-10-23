use std::io;
use std::process::Command;

use super::{
    GitOperations,
    GitTestConstants,
    TestDir,
};

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

    fn init_repo(&self, test_dir: &TestDir) -> io::Result<()> {
        self.init_repo_no_commit(test_dir)?;
        test_dir.create_file(
            GitTestConstants::INITIAL_FILE_NAME,
            GitTestConstants::INITIAL_FILE_CONTENT,
        )?;
        self.create_commit(test_dir, GitTestConstants::INITIAL_COMMIT_MESSAGE)?;
        Ok(())
    }

    fn init_repo_no_commit(&self, test_dir: &TestDir) -> io::Result<()> {
        self.execute_git(test_dir, &["init", "-b", GitTestConstants::DEFAULT_BRANCH])?;
        self.execute_git(
            test_dir,
            &["config", "user.name", GitTestConstants::TEST_USER_NAME],
        )?;
        self.execute_git(
            test_dir,
            &["config", "user.email", GitTestConstants::TEST_USER_EMAIL],
        )?;
        Ok(())
    }

    fn create_commit(&self, test_dir: &TestDir, message: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["add", "."])?;
        self.execute_git(test_dir, &["commit", "-m", message])?;
        Ok(())
    }

    fn create_tag(&self, test_dir: &TestDir, tag: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["tag", tag])?;
        Ok(())
    }
}
