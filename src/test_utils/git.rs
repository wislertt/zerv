use super::TestDir;
use std::io;
use std::process::Command;

/// Docker-based git operations for integration testing
#[derive(Default)]
pub struct DockerGit;

impl DockerGit {
    pub fn new() -> Self {
        Self
    }

    fn run_docker_command(&self, test_dir: &TestDir, script: &str) -> io::Result<String> {
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--entrypoint",
                "sh",
                "-v",
                &format!("{}:/workspace", test_dir.path().display()),
                "-w",
                "/workspace",
                "--user",
                "root",
                "alpine/git:latest",
                "-c",
                script,
            ])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Docker command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn run_git_command(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
        let git_command = args
            .iter()
            .map(|arg| {
                if arg.contains(' ') {
                    format!("'{arg}'")
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        self.run_docker_command(test_dir, &format!("git {git_command}"))
    }

    pub fn init_repo(&self, test_dir: &TestDir) -> io::Result<()> {
        // Create initial file and setup repo in single command to avoid race conditions
        test_dir.create_file("README.md", "# Test Repository")?;
        let init_script = [
            "git init -b main",
            "git config user.name 'Test User'",
            "git config user.email 'test@example.com'",
            "git add .",
            "git commit -m 'Initial commit'",
        ]
        .join(" && ");

        self.run_docker_command(test_dir, &init_script)?;
        Ok(())
    }

    pub fn create_commit(&self, test_dir: &TestDir, message: &str) -> io::Result<()> {
        self.run_docker_command(test_dir, &format!("git add . && git commit -m '{message}'"))?;
        Ok(())
    }

    pub fn create_tag(&self, test_dir: &TestDir, tag: &str) -> io::Result<()> {
        self.run_docker_command(test_dir, &format!("git tag {tag}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // Error message constants
    const DOCKER_INIT_ERROR: &str = "Docker git init should succeed";
    const DOCKER_COMMIT_ERROR: &str = "Docker git commit should succeed";
    const DOCKER_TAG_ERROR: &str = "Docker git tag should succeed";
    const TEST_DIR_ERROR: &str = "Failed to create test dir";

    #[rstest]
    #[case(DOCKER_INIT_ERROR)]
    #[case(DOCKER_COMMIT_ERROR)]
    #[case(DOCKER_TAG_ERROR)]
    #[case(TEST_DIR_ERROR)]
    fn test_error_message_constants(#[case] message: &str) {
        assert!(!message.is_empty());
        assert!(message.len() > 10);
    }

    fn setup_docker_git() -> (TestDir, DockerGit) {
        let dir = TestDir::new().expect(TEST_DIR_ERROR);
        let docker_git = DockerGit::new();
        (dir, docker_git)
    }

    fn setup_initialized_repo() -> (TestDir, DockerGit) {
        let (dir, docker_git) = setup_docker_git();
        docker_git.init_repo(&dir).expect(DOCKER_INIT_ERROR);
        (dir, docker_git)
    }

    fn setup_repo_with_commit() -> (TestDir, DockerGit) {
        // setup_initialized_repo already creates a commit, so just return it
        setup_initialized_repo()
    }

    #[test]
    fn test_docker_git_new() {
        let docker_git = DockerGit::new();
        assert!(std::mem::size_of_val(&docker_git) == 0);
    }

    #[test]
    fn test_setup_functions() {
        let (dir, _docker_git) = setup_docker_git();
        assert!(dir.path().exists());
    }

    fn is_docker_available() -> bool {
        Command::new("docker")
            .args(["run", "--rm", "alpine/git:latest", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_is_docker_available() {
        let _result = is_docker_available();
    }

    #[rstest]
    #[case(&["--version"])]
    #[case(&["status"])]
    #[case(&["log", "--oneline"])]
    fn test_docker_git_commands_without_docker(#[case] args: &[&str]) {
        let (dir, docker_git) = setup_docker_git();
        let result = docker_git.run_git_command(&dir, args);
        let _ = result;
    }

    #[test]
    fn test_docker_git_init_repo_without_docker() {
        let (dir, docker_git) = setup_docker_git();
        let result = docker_git.init_repo(&dir);
        let _ = result;
    }

    #[test]
    fn test_docker_git_create_commit_without_docker() {
        let (dir, docker_git) = setup_docker_git();
        dir.create_file("test.txt", "content").unwrap();
        let result = docker_git.create_commit(&dir, "test commit");
        let _ = result;
    }

    #[test]
    fn test_docker_git_create_tag_without_docker() {
        let (dir, docker_git) = setup_docker_git();
        let result = docker_git.create_tag(&dir, "v1.0.0");
        let _ = result;
    }

    #[test]
    #[ignore = "docker"]
    fn test_setup_initialized_repo_without_docker() {
        let result = std::panic::catch_unwind(|| {
            let (_dir, _docker_git) = setup_initialized_repo();
        });
        let _ = result;
    }

    #[test]
    #[ignore = "docker"]
    fn test_setup_repo_with_commit_without_docker() {
        let result = std::panic::catch_unwind(|| {
            let (_dir, _docker_git) = setup_repo_with_commit();
        });
        let _ = result;
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_init() {
        let (dir, _docker_git) = setup_initialized_repo();
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_commit() {
        let (dir, docker_git) = setup_initialized_repo();
        dir.create_file("test.txt", "test content").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect(DOCKER_COMMIT_ERROR);
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_tag() {
        let (dir, docker_git) = setup_repo_with_commit();
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect(DOCKER_TAG_ERROR);
    }

    #[test]
    #[ignore = "docker"]
    fn test_docker_git_integration() {
        let (dir, docker_git) = setup_repo_with_commit();
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect(DOCKER_TAG_ERROR);
        dir.create_file("feature.txt", "new feature").unwrap();
        docker_git
            .create_commit(&dir, "Add feature")
            .expect(DOCKER_COMMIT_ERROR);

        assert!(dir.path().join(".git").exists());
        assert!(dir.path().join("README.md").exists());
        assert!(dir.path().join("feature.txt").exists());
    }
}
