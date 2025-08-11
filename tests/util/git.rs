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

/// Docker-based git operations for integration testing
struct DockerGit;

impl DockerGit {
    fn new() -> Self {
        Self
    }

    fn run_git_command(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
        let output = Command::new("/opt/homebrew/bin/docker")
            .args([
                "run",
                "--rm",
                "-v",
                &format!("{}:/workspace", test_dir.path().display()),
                "-w",
                "/workspace",
                "alpine/git:latest",
            ])
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Docker git command failed: {}",
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

    // Docker-based integration tests - ignored by default
    #[test]
    #[ignore = "docker"]
    fn test_docker_git_init() {
        let dir = TestDir::new().expect("Failed to create test dir");
        let docker_git = DockerGit::new();

        docker_git
            .init_repo(&dir)
            .expect("Docker git init should succeed");
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_commit() {
        let dir = TestDir::new().expect("Failed to create test dir");
        let docker_git = DockerGit::new();

        docker_git
            .init_repo(&dir)
            .expect("Docker git init should succeed");
        dir.create_file("test.txt", "test content").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect("Docker git commit should succeed");
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_tag() {
        let dir = TestDir::new().expect("Failed to create test dir");
        let docker_git = DockerGit::new();

        docker_git
            .init_repo(&dir)
            .expect("Docker git init should succeed");
        dir.create_file("README.md", "# Test").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect("Docker git commit should succeed");
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect("Docker git tag should succeed");
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_integration() {
        let dir = TestDir::new().expect("Failed to create test dir");
        let docker_git = DockerGit::new();

        // Test full workflow
        docker_git
            .init_repo(&dir)
            .expect("Docker git init should succeed");
        dir.create_file("README.md", "# Test Repository").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect("Docker git commit should succeed");
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect("Docker git tag should succeed");
        dir.create_file("feature.txt", "new feature").unwrap();
        docker_git
            .create_commit(&dir, "Add feature")
            .expect("Docker git second commit should succeed");

        // Verify files exist
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join("README.md").exists());
        assert!(dir.path().join("feature.txt").exists());
    }
}
