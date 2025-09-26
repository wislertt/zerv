use super::{GitOperations, TestDir};
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

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

/// Docker-based git operations for integration testing with container reuse optimization
///
/// This implementation reuses a single long-running Docker container
/// to avoid the overhead of creating new containers for each Git operation.
pub struct DockerGit {
    container_id: Arc<Mutex<Option<String>>>,
    current_test_dir: Arc<Mutex<Option<PathBuf>>>,
}

impl Default for DockerGit {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerGit {
    pub fn new() -> Self {
        Self {
            container_id: Arc::new(Mutex::new(None)),
            current_test_dir: Arc::new(Mutex::new(None)),
        }
    }

    fn ensure_container_running(&self, test_dir: &TestDir) -> io::Result<()> {
        let test_dir_path = test_dir.path().to_path_buf();

        // Check if we need to start a new container
        let needs_new_container = {
            let container_id = self.container_id.lock().unwrap();
            let current_dir = self.current_test_dir.lock().unwrap();

            container_id.is_none() || current_dir.as_ref() != Some(&test_dir_path)
        };

        if needs_new_container {
            self.start_container(test_dir)?;
        }

        Ok(())
    }

    fn start_container(&self, test_dir: &TestDir) -> io::Result<()> {
        // Clean up existing container if any
        self.cleanup_container()?;

        // Use current user on Unix systems, root on others
        #[cfg(unix)]
        let user_args = {
            let uid = unsafe { libc::getuid() };
            let gid = unsafe { libc::getgid() };
            vec!["--user".to_string(), format!("{uid}:{gid}")]
        };
        #[cfg(not(unix))]
        let user_args: Vec<String> = vec![];

        let mut args = vec![
            "run",
            "-d", // Run in detached mode for container reuse
            "--security-opt=no-new-privileges",
            "--cap-drop=ALL",
        ];

        // Add user args if present
        for arg in &user_args {
            args.push(arg);
        }

        let volume_mount = format!("{}:/workspace", test_dir.path().display());
        args.extend([
            "--entrypoint",
            "sh",
            "-v",
            &volume_mount,
            "-w",
            "/workspace",
            "alpine/git:latest",
            "-c",
            "while true; do sleep 30; done", // Keep container alive
        ]);

        #[cfg(test)]
        if let Err(e) = validate_docker_args(&args) {
            return Err(io::Error::other(e));
        }

        let output = Command::new("docker").args(args).output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Failed to start Docker container: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Update state
        {
            let mut container_id_guard = self.container_id.lock().unwrap();
            let mut current_dir_guard = self.current_test_dir.lock().unwrap();
            *container_id_guard = Some(container_id);
            *current_dir_guard = Some(test_dir.path().to_path_buf());
        }

        Ok(())
    }

    fn cleanup_container(&self) -> io::Result<()> {
        let container_id = {
            let mut container_id_guard = self.container_id.lock().unwrap();
            container_id_guard.take()
        };

        if let Some(id) = container_id {
            let _ = Command::new("docker").args(["rm", "-f", &id]).output(); // Ignore errors during cleanup
        }

        Ok(())
    }

    fn run_docker_command(&self, test_dir: &TestDir, script: &str) -> io::Result<String> {
        let start = std::time::Instant::now();

        self.ensure_container_running(test_dir)?;

        let container_id = {
            let container_id_guard = self.container_id.lock().unwrap();
            container_id_guard.clone().unwrap()
        };

        let output = Command::new("docker")
            .args(["exec", &container_id, "sh", "-c", script])
            .output()?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "Docker exec command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let duration = start.elapsed();
        eprintln!("🐳 Docker Git command '{script}' took {duration:?}");

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

impl Drop for DockerGit {
    fn drop(&mut self) {
        let _ = self.cleanup_container();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::should_run_docker_tests;
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
        // DockerGit now contains Arc<Mutex<>> fields for container management
        // so it's no longer zero-sized, but should still be relatively small
        assert!(std::mem::size_of_val(&docker_git) > 0);
        assert!(std::mem::size_of_val(&docker_git) < 100); // Reasonable upper bound
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
        if should_run_docker_tests() {
            assert!(
                is_docker_available(),
                "ZERV_TEST_DOCKER is enabled but Docker is not available - install Docker or disable ZERV_TEST_DOCKER"
            );
        } else {
            assert!(
                !is_docker_available(),
                "Docker is available but ZERV_TEST_DOCKER is disabled - enable ZERV_TEST_DOCKER to test Docker functionality"
            );
        }
    }

    #[rstest]
    #[case(&["--version"])]
    #[case(&["status"])]
    #[case(&["log", "--oneline"])]
    fn test_docker_git_commands_without_docker(#[case] args: &[&str]) {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }
        let (dir, docker_git) = setup_docker_git();
        let result = docker_git.run_git_command(&dir, args);
        let _ = result;
    }

    #[test]
    fn test_docker_git_init_repo_without_docker() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }
        let (dir, docker_git) = setup_docker_git();
        let result = docker_git.init_repo(&dir);
        let _ = result;
    }

    #[test]
    fn test_docker_git_create_commit_without_docker() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }
        let (dir, docker_git) = setup_docker_git();
        dir.create_file("test.txt", "content").unwrap();
        let result = docker_git.create_commit(&dir, "test commit");
        let _ = result;
    }

    #[test]
    fn test_docker_git_create_tag_without_docker() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }
        let (dir, docker_git) = setup_docker_git();
        let result = docker_git.create_tag(&dir, "v1.0.0");
        let _ = result;
    }

    #[test]
    fn test_setup_initialized_repo_without_docker() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }
        let (_dir, _docker_git) = setup_initialized_repo();
    }

    #[test]
    fn test_setup_repo_with_commit_without_docker() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }
        let (_dir, _docker_git) = setup_repo_with_commit();
    }

    #[test]
    fn test_docker_git_init() {
        if !should_run_docker_tests() {
            return;
        }
        let (dir, _docker_git) = setup_initialized_repo();
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    fn test_docker_git_commit() {
        if !should_run_docker_tests() {
            return;
        }
        let (dir, docker_git) = setup_initialized_repo();
        dir.create_file("test.txt", "test content").unwrap();
        docker_git
            .create_commit(&dir, "Initial commit")
            .expect(DOCKER_COMMIT_ERROR);
    }

    #[test]
    fn test_docker_git_tag() {
        if !should_run_docker_tests() {
            return;
        }
        let (dir, docker_git) = setup_repo_with_commit();
        docker_git
            .create_tag(&dir, "v1.0.0")
            .expect(DOCKER_TAG_ERROR);
    }

    #[test]
    fn test_docker_git_integration() {
        if !should_run_docker_tests() {
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
