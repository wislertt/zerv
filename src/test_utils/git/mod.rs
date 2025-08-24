use super::TestDir;
use std::io;

mod docker;
mod native;

pub use docker::DockerGit;
pub use native::NativeGit;

/// Common Git operations trait for both Docker and Native implementations
pub trait GitOperations {
    /// Execute a git command with the given arguments
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String>;

    /// Initialize a git repository with initial commit (shared logic)
    fn init_repo(&self, test_dir: &TestDir) -> io::Result<()> {
        test_dir.create_file("README.md", "# Test Repository")?;
        self.execute_git(test_dir, &["init"])?;
        self.execute_git(test_dir, &["config", "user.name", "Test User"])?;
        self.execute_git(test_dir, &["config", "user.email", "test@example.com"])?;
        self.execute_git(test_dir, &["add", "."])?;
        self.execute_git(test_dir, &["commit", "-m", "Initial commit"])?;
        Ok(())
    }

    /// Create a git tag (shared logic)
    fn create_tag(&self, test_dir: &TestDir, tag: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["tag", tag])?;
        Ok(())
    }

    /// Create a commit (shared logic)
    fn create_commit(&self, test_dir: &TestDir, message: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["add", "."])?;
        self.execute_git(test_dir, &["commit", "-m", message])?;
        Ok(())
    }
}
