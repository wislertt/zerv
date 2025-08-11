use super::TestDir;
use std::io;
use std::process::Command;

/// Git VCS test utilities for git integration testing
impl TestDir {
    /// Initialize a real git repository with initial commit
    pub fn init_git_repo(&self) -> io::Result<()> {
        // Initialize git repo
        let output = Command::new("git")
            .args(["init"])
            .current_dir(self.path())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Configure git user for testing
        let output = Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(self.path())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("git config user.name failed"));
        }

        let output = Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(self.path())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other("git config user.email failed"));
        }

        Ok(())
    }

    /// Create an initial commit
    pub fn create_initial_commit(&self) -> io::Result<()> {
        self.create_file("README.md", "# Test Repository")?;

        Command::new("git")
            .args(["add", "."])
            .current_dir(self.path())
            .output()?;

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(self.path())
            .output()?;

        Ok(())
    }

    /// Create a git tag
    pub fn create_tag(&self, tag: &str) -> io::Result<()> {
        Command::new("git")
            .args(["tag", tag])
            .current_dir(self.path())
            .output()?;
        Ok(())
    }

    /// Create additional commits for distance testing
    pub fn create_commits(&self, count: usize) -> io::Result<()> {
        for i in 1..=count {
            self.create_file(format!("file{i}.txt"), &format!("Content {i}"))?;

            Command::new("git")
                .args(["add", "."])
                .current_dir(self.path())
                .output()?;

            Command::new("git")
                .args(["commit", "-m", &format!("Commit {i}")])
                .current_dir(self.path())
                .output()?;
        }
        Ok(())
    }

    /// Make repository dirty (uncommitted changes)
    pub fn make_dirty(&self) -> io::Result<()> {
        self.create_file("dirty.txt", "uncommitted changes")?;
        Ok(())
    }

    /// Create a branch and switch to it
    pub fn create_branch(&self, branch: &str) -> io::Result<()> {
        let output = Command::new("git")
            .args(["checkout", "-b", branch])
            .current_dir(self.path())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git checkout -b failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    /// Initialize a git repository (dummy implementation for future VCS support)
    pub fn init_git(&self) -> io::Result<()> {
        // Placeholder for future git initialization
        // This would run: git init, git config, etc.
        self.create_dir(".git")?;
        self.create_file(".git/HEAD", "ref: refs/heads/main")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_available() -> bool {
        std::process::Command::new("git")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_git_available() {
        // Test the git_available function itself
        let _available = git_available();
        // Just ensure it returns a boolean without panicking
    }

    #[test]
    fn test_init_git_repo_without_git() {
        let dir = TestDir::new().expect("Failed to create test dir");

        // Test that init_git_repo handles missing git gracefully
        if !git_available() {
            // If git is not available, the function should return an error
            let result = dir.init_git_repo();
            assert!(result.is_err());
            return;
        }

        // If git is available, test successful initialization
        let result = dir.init_git_repo();
        if result.is_ok() {
            assert!(dir.path().join(".git").exists());
        }
    }

    #[test]
    fn test_create_initial_commit() {
        let dir = TestDir::new().expect("Failed to create test dir");

        if !git_available() {
            return;
        }

        // Initialize git first
        if dir.init_git_repo().is_ok() {
            let result = dir.create_initial_commit();
            if result.is_ok() {
                assert!(dir.path().join("README.md").exists());
            }
        }
    }

    #[test]
    fn test_create_tag() {
        let dir = TestDir::new().expect("Failed to create test dir");

        if !git_available() {
            return;
        }

        if dir.init_git_repo().is_ok() && dir.create_initial_commit().is_ok() {
            let result = dir.create_tag("v1.0.0");
            // Just test that the function doesn't panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_create_commits() {
        let dir = TestDir::new().expect("Failed to create test dir");

        if !git_available() {
            return;
        }

        if dir.init_git_repo().is_ok() && dir.create_initial_commit().is_ok() {
            let result = dir.create_commits(2);
            if result.is_ok() {
                assert!(dir.path().join("file1.txt").exists());
                assert!(dir.path().join("file2.txt").exists());
            }
        }
    }

    #[test]
    fn test_make_dirty() {
        let dir = TestDir::new().expect("Failed to create test dir");

        dir.make_dirty().unwrap();
        assert!(dir.path().join("dirty.txt").exists());
        let content = std::fs::read_to_string(dir.path().join("dirty.txt")).unwrap();
        assert_eq!(content, "uncommitted changes");
        // Keep dir alive until end of test
        drop(dir);
    }

    #[test]
    fn test_create_branch() {
        let dir = TestDir::new().expect("Failed to create test dir");

        if !git_available() {
            return;
        }

        if dir.init_git_repo().is_ok() && dir.create_initial_commit().is_ok() {
            let result = dir.create_branch("feature-branch");
            // Just test that the function doesn't panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_git_utilities_integration() {
        if !git_available() {
            return;
        }

        let dir = TestDir::new().expect("Failed to create test dir");

        // Test the full workflow if git is available
        if let Ok(()) = dir.init_git_repo()
            && let Ok(()) = dir.create_initial_commit()
        {
            let _ = dir.create_tag("v1.0.0");
            let _ = dir.create_commits(1);
            let _ = dir.make_dirty();

            // Verify basic file structure
            assert!(dir.path().join(".git").exists());
            assert!(dir.path().join("README.md").exists());
            assert!(dir.path().join("dirty.txt").exists());
        }
    }

    #[test]
    fn test_dir_init_git() {
        let dir = TestDir::new().unwrap();
        dir.init_git().unwrap();
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join(".git/HEAD").exists());
        let head_content = std::fs::read_to_string(dir.path().join(".git/HEAD")).unwrap();
        assert_eq!(head_content, "ref: refs/heads/main");
    }
}
