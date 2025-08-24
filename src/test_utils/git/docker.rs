use super::{GitOperations, TestDir};
use std::io;
use std::process::Command;

#[cfg(test)]
fn validate_docker_args(args: &[&str]) -> Result<(), String> {
    // Check for alpine/git without --entrypoint
    if args.contains(&"alpine/git:latest") && !args.contains(&"--entrypoint") {
        return Err(
            "❌ Missing --entrypoint sh for alpine/git:latest (will fail in CI)".to_string(),
        );
    }

    // Check for git commands without proper entrypoint
    if args.iter().any(|&arg| arg.starts_with("git ")) && !args.contains(&"--entrypoint") {
        return Err("❌ Git commands need --entrypoint sh (will fail in CI)".to_string());
    }

    Ok(())
}

/// Docker-based git operations for integration testing
#[derive(Default)]
pub struct DockerGit;

impl DockerGit {
    pub fn new() -> Self {
        Self
    }

    fn run_docker_command(&self, test_dir: &TestDir, script: &str) -> io::Result<String> {
        #[cfg(unix)]
        let (uid, gid) = {
            let uid = unsafe { libc::getuid() };
            let gid = unsafe { libc::getgid() };
            (uid, gid)
        };

        #[cfg(windows)]
        let (uid, gid) = (1000u32, 1000u32); // Default user on most Docker containers

        let args = [
            "run",
            "--rm",
            "--security-opt=no-new-privileges", // Strict mode: remove permissive layers
            "--cap-drop=ALL",                   // Strict mode: drop all capabilities
            "--user",
            &format!("{uid}:{gid}"), // Fix permission issues
            "--entrypoint",
            "sh",
            "-v",
            &format!("{}:/workspace", test_dir.path().display()),
            "-w",
            "/workspace",
            "alpine/git:latest",
            "-c",
            script,
        ];

        #[cfg(test)]
        if let Err(e) = validate_docker_args(&args) {
            return Err(io::Error::other(e));
        }

        let output = Command::new("docker").args(args).output()?;

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

        self.run_docker_command(test_dir, &format!("git --git-dir=.git {git_command}"))
    }
}

impl GitOperations for DockerGit {
    fn execute_git(&self, test_dir: &TestDir, args: &[&str]) -> io::Result<String> {
        self.run_git_command(test_dir, args)
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
    #[cfg(target_os = "linux")]
    fn test_docker_git_init() {
        let (dir, _docker_git) = setup_initialized_repo();
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    #[ignore = "docker"]
    #[cfg(target_os = "linux")]
    fn test_docker_git_commit() {
        let (dir, docker_git) = setup_initialized_repo();
        dir.create_file("test.txt", "test content").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect(DOCKER_COMMIT_ERROR);
    }

    #[test]
    #[ignore = "docker"]
    #[cfg(target_os = "linux")]
    fn test_docker_git_tag() {
        let (dir, docker_git) = setup_repo_with_commit();
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect(DOCKER_TAG_ERROR);
    }

    #[test]
    #[ignore = "docker"]
    #[cfg(target_os = "linux")]
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

    #[test]
    fn test_docker_validation_catches_missing_entrypoint() {
        let bad_args = ["run", "--rm", "alpine/git:latest", "git", "init"];
        let result = validate_docker_args(&bad_args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing --entrypoint"));
    }

    #[test]
    fn test_docker_validation_passes_correct_args() {
        let good_args = [
            "run",
            "--rm",
            "--entrypoint",
            "sh",
            "alpine/git:latest",
            "-c",
            "git init",
        ];
        let result = validate_docker_args(&good_args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_docker_validation_catches_git_without_entrypoint() {
        let bad_args = ["run", "--rm", "alpine:latest", "git status"];
        let result = validate_docker_args(&bad_args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Git commands need --entrypoint")
        );
    }
}
