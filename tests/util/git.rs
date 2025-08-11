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
        let output = Command::new("docker")
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

    // Error message constants
    const DOCKER_INIT_ERROR: &str = "Docker git init should succeed";
    const DOCKER_COMMIT_ERROR: &str = "Docker git commit should succeed";
    const DOCKER_TAG_ERROR: &str = "Docker git tag should succeed";
    const TEST_DIR_ERROR: &str = "Failed to create test dir";

    // Helper for Docker test setup
    fn setup_docker_git() -> (TestDir, DockerGit) {
        let dir = TestDir::new().expect(TEST_DIR_ERROR);
        let docker_git = DockerGit::new();
        (dir, docker_git)
    }

    // Helper for initialized Docker git repo
    fn setup_initialized_repo() -> (TestDir, DockerGit) {
        let (dir, docker_git) = setup_docker_git();
        docker_git.init_repo(&dir).expect(DOCKER_INIT_ERROR);
        (dir, docker_git)
    }

    // Helper for repo with initial commit
    fn setup_repo_with_commit() -> (TestDir, DockerGit) {
        let (dir, docker_git) = setup_initialized_repo();
        dir.create_file("README.md", "# Test").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect(DOCKER_COMMIT_ERROR);
        (dir, docker_git)
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

    // Helper to check if Docker is available
    fn is_docker_available() -> bool {
        Command::new("docker")
            .args(["--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    // Docker-based integration tests - ignored by default
    #[test]
    #[ignore = "docker"]
    fn test_docker_git_init() {
        if !is_docker_available() {
            eprintln!("Docker not available, skipping test");
            return;
        }
        let (dir, docker_git) = setup_docker_git();
        docker_git.init_repo(&dir).expect(DOCKER_INIT_ERROR);
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_commit() {
        if !is_docker_available() {
            eprintln!("Docker not available, skipping test");
            return;
        }
        let (dir, docker_git) = setup_initialized_repo();
        dir.create_file("test.txt", "test content").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect(DOCKER_COMMIT_ERROR);
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_tag() {
        if !is_docker_available() {
            eprintln!("Docker not available, skipping test");
            return;
        }
        let (dir, docker_git) = setup_repo_with_commit();
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect(DOCKER_TAG_ERROR);
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_integration() {
        if !is_docker_available() {
            eprintln!("Docker not available, skipping test");
            return;
        }
        let (dir, docker_git) = setup_repo_with_commit();
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect(DOCKER_TAG_ERROR);
        dir.create_file("feature.txt", "new feature").unwrap();
        docker_git
            .create_commit(&dir, "Add feature")
            .expect(DOCKER_COMMIT_ERROR);

        // Verify files exist
        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join("README.md").exists());
        assert!(dir.path().join("feature.txt").exists());
    }
}
