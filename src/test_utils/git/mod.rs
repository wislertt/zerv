use std::io;

use super::TestDir;

mod docker;
mod fixtures;
mod native;

pub use docker::DockerGit;
pub use fixtures::GitRepoFixture;
pub use native::NativeGit;

/// Constants for Git test operations to ensure consistency across implementations
pub struct GitTestConstants;

impl GitTestConstants {
    /// Default filename for the initial repository file
    pub const INITIAL_FILE_NAME: &'static str = "README.md";

    /// Default content for the initial repository file
    pub const INITIAL_FILE_CONTENT: &'static str = "# Test Repository";

    /// Default message for the initial commit
    pub const INITIAL_COMMIT_MESSAGE: &'static str = "Initial commit";

    /// Default Git user name for test repositories
    pub const TEST_USER_NAME: &'static str = "Test User";

    /// Default Git user email for test repositories
    pub const TEST_USER_EMAIL: &'static str = "test@example.com";

    /// Default branch name for test repositories
    pub const DEFAULT_BRANCH: &'static str = "main";
}

/// Common Git operations trait for both Docker and Native implementations
pub trait GitOperations {
    /// Execute a git command with the given arguments
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String>;

    /// Initialize a git repository with initial commit
    fn init_repo(&self, test_dir: &TestDir) -> io::Result<()>;

    /// Initialize an empty git repository without any commits
    fn init_repo_no_commit(&self, test_dir: &TestDir) -> io::Result<()>;

    /// Create a git tag
    fn create_tag(&self, test_dir: &TestDir, tag: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["tag", tag])?;
        Ok(())
    }

    /// Create a commit
    fn create_commit(&self, test_dir: &TestDir, message: &str) -> io::Result<()>;

    /// Create and checkout a new branch
    fn create_branch(&self, test_dir: &TestDir, branch_name: &str) -> io::Result<()> {
        self.execute_git(test_dir, &["checkout", "-b", branch_name])?;
        Ok(())
    }
}
