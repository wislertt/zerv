use super::TestDir;
use std::io;
use std::process::Command;

/// Git VCS test utilities
impl TestDir {
    /// Initialize a dummy git repository (no real git commands)
    pub fn init_git(&self) -> io::Result<()> {
        self.create_dir(".git")?;
        self.create_file(".git/HEAD", "ref: refs/heads/main")?;
        self.create_dir(".git/refs/heads")?;
        self.create_file(".git/refs/heads/main", "dummy-commit-hash")?;
        Ok(())
    }

    /// Create dummy git files for testing
    pub fn create_dummy_git_files(&self) -> io::Result<()> {
        self.init_git()?;
        self.create_file("README.md", "# Test Repository")?;
        Ok(())
    }
}

/// System git operations for integration testing
struct SystemGit;

impl SystemGit {
    fn new() -> Self {
        Self
    }

    fn run_git_command(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
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

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn init_repo(&self, test_dir: &TestDir) -> io::Result<()> {
        self.run_git_command(test_dir, &["init"])?;
        self.run_git_command(test_dir, &["config", "user.name", "Test User"])?;
        self.run_git_command(test_dir, &["config", "user.email", "test@example.com"])?;
        Ok(())
    }

    fn create_commit(&self, test_dir: &TestDir, message: &str) -> io::Result<()> {
        self.run_git_command(test_dir, &["add", "."])?;
        self.run_git_command(test_dir, &["commit", "-m", message])?;
        Ok(())
    }

    fn create_tag(&self, test_dir: &TestDir, tag: &str) -> io::Result<()> {
        self.run_git_command(test_dir, &["tag", tag])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Error message constants
    const GIT_INIT_ERROR: &str = "Git init should succeed";
    const GIT_COMMIT_ERROR: &str = "Git commit should succeed";
    const GIT_TAG_ERROR: &str = "Git tag should succeed";
    const TEST_DIR_ERROR: &str = "Failed to create test dir";

    // Helper for git test setup
    fn setup_system_git() -> (TestDir, SystemGit) {
        let dir = TestDir::new().expect(TEST_DIR_ERROR);
        let system_git = SystemGit::new();
        (dir, system_git)
    }

    // Helper for initialized git repo
    fn setup_initialized_repo() -> (TestDir, SystemGit) {
        let (dir, system_git) = setup_system_git();
        system_git.init_repo(&dir).expect(GIT_INIT_ERROR);
        (dir, system_git)
    }

    // Helper for repo with initial commit
    fn setup_repo_with_commit() -> (TestDir, SystemGit) {
        let (dir, system_git) = setup_initialized_repo();
        dir.create_file("README.md", "# Test").unwrap();
        system_git
            .create_commit(&dir, "Initial commit")
            .expect(GIT_COMMIT_ERROR);
        (dir, system_git)
    }

    // Fast tests - always run (no Docker required)
    #[test]
    fn test_dummy_git_structure() {
        let dir = TestDir::new().unwrap();
        dir.init_git().unwrap();
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join(".git/HEAD").exists());
        let head_content = std::fs::read_to_string(dir.path().join(".git/HEAD")).unwrap();
        assert_eq!(head_content, "ref: refs/heads/main");
    }

    #[test]
    fn test_dummy_git_files() {
        let dir = TestDir::new().unwrap();
        dir.create_dummy_git_files().unwrap();
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join("README.md").exists());
    }

    // Helper to check if git is available
    fn is_git_available() -> bool {
        Command::new("git")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    // Git-based integration tests
    #[test]
    fn test_system_git_init() {
        if !is_git_available() {
            eprintln!("Git not available, skipping test");
            return;
        }
        let (dir, system_git) = setup_system_git();
        system_git.init_repo(&dir).expect(GIT_INIT_ERROR);
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    fn test_system_git_commit() {
        if !is_git_available() {
            eprintln!("Git not available, skipping test");
            return;
        }
        let (dir, system_git) = setup_initialized_repo();
        dir.create_file("test.txt", "test content").unwrap();
        system_git
            .create_commit(&dir, "Initial commit")
            .expect(GIT_COMMIT_ERROR);
    }

    #[test]
    fn test_system_git_tag() {
        if !is_git_available() {
            eprintln!("Git not available, skipping test");
            return;
        }
        let (dir, system_git) = setup_repo_with_commit();
        system_git.create_tag(&dir, "v1.0.0").expect(GIT_TAG_ERROR);
    }

    #[test]
    fn test_system_git_integration() {
        if !is_git_available() {
            eprintln!("Git not available, skipping test");
            return;
        }
        let (dir, system_git) = setup_repo_with_commit();
        system_git.create_tag(&dir, "v1.0.0").expect(GIT_TAG_ERROR);
        dir.create_file("feature.txt", "new feature").unwrap();
        system_git
            .create_commit(&dir, "Add feature")
            .expect(GIT_COMMIT_ERROR);

        // Verify files exist
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join("README.md").exists());
        assert!(dir.path().join("feature.txt").exists());
    }
}
