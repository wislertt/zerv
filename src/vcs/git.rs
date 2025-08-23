use crate::error::{Result, ZervError};
use crate::vcs::{Vcs, VcsData};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git VCS implementation
pub struct GitVcs {
    repo_path: PathBuf,
    // TODO: Add optional tag_branch parameter for future extension
    // tag_branch: Option<String>,
}

impl GitVcs {
    /// Create new Git VCS instance
    pub fn new(path: &Path) -> Result<Self> {
        let repo_path = crate::vcs::find_vcs_root(path)?;
        Ok(Self { repo_path })
    }

    /// Run git command and return output
    fn run_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| ZervError::CommandFailed(format!("Failed to execute git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ZervError::CommandFailed(format!(
                "Git command failed: {stderr}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get latest version tag
    fn get_latest_tag(&self) -> Result<Option<String>> {
        match self.run_git_command(&["describe", "--tags", "--abbrev=0"]) {
            Ok(tag) if !tag.is_empty() => Ok(Some(tag)),
            Ok(_) => Ok(None),
            Err(ZervError::CommandFailed(_)) => Ok(None), // No tags found
            Err(e) => Err(e),
        }
    }

    /// Calculate distance from tag to HEAD
    fn calculate_distance(&self, tag: &str) -> Result<u32> {
        let output = self.run_git_command(&["rev-list", "--count", &format!("{tag}..HEAD")])?;
        output
            .parse::<u32>()
            .map_err(|e| ZervError::CommandFailed(format!("Failed to parse distance: {e}")))
    }

    /// Get current commit hash (full)
    fn get_commit_hash(&self) -> Result<String> {
        self.run_git_command(&["rev-parse", "HEAD"])
    }

    /// Get current commit hash (short)
    fn get_commit_hash_short(&self) -> Result<String> {
        self.run_git_command(&["rev-parse", "--short", "HEAD"])
    }

    /// Get current branch name
    fn get_current_branch(&self) -> Result<Option<String>> {
        match self.run_git_command(&["branch", "--show-current"]) {
            Ok(branch) if !branch.is_empty() => Ok(Some(branch)),
            Ok(_) => Ok(None), // Detached HEAD
            Err(_) => Ok(None),
        }
    }

    /// Get commit timestamp
    fn get_commit_timestamp(&self) -> Result<i64> {
        let output = self.run_git_command(&["log", "-1", "--format=%ct"])?;
        output
            .parse::<i64>()
            .map_err(|e| ZervError::CommandFailed(format!("Failed to parse timestamp: {e}")))
    }

    /// Get tag timestamp
    fn get_tag_timestamp(&self, tag: &str) -> Result<Option<i64>> {
        // Check if tag is annotated or lightweight
        let tag_type = match self.run_git_command(&["cat-file", "-t", tag]) {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };

        let timestamp = match tag_type.trim() {
            "tag" => {
                // Annotated tag - use tag creation date
                self.run_git_command(&["log", "-1", "--format=%ct", tag])?
            }
            "commit" => {
                // Lightweight tag - use commit date
                self.run_git_command(&["log", "-1", "--format=%ct", tag])?
            }
            _ => return Ok(None),
        };

        timestamp
            .parse::<i64>()
            .map(Some)
            .map_err(|e| ZervError::CommandFailed(format!("Failed to parse tag timestamp: {e}")))
    }

    /// Check if working directory is dirty
    fn is_dirty(&self) -> Result<bool> {
        let output = self.run_git_command(&["status", "--porcelain"])?;
        Ok(!output.is_empty())
    }

    /// Check for shallow clone and warn user
    fn check_shallow_clone(&self) -> bool {
        self.repo_path.join(".git/shallow").exists()
    }
}

impl Vcs for GitVcs {
    fn get_vcs_data(&self) -> Result<VcsData> {
        // Check for shallow clone and warn
        if self.check_shallow_clone() {
            eprintln!("Warning: Shallow clone detected - distance calculations may be inaccurate");
        }

        let mut data = VcsData {
            commit_hash: self.get_commit_hash()?,
            commit_hash_short: self.get_commit_hash_short()?,
            commit_timestamp: self.get_commit_timestamp()?,
            is_dirty: self.is_dirty()?,
            current_branch: self.get_current_branch().unwrap_or(None),
            ..Default::default()
        };

        // Get tag information
        if let Some(tag) = self.get_latest_tag()? {
            data.distance = self.calculate_distance(&tag).unwrap_or(0);
            data.tag_timestamp = self.get_tag_timestamp(&tag).unwrap_or(None);
            data.tag_version = Some(tag);
        }

        Ok(data)
    }

    fn is_available(&self, path: &Path) -> bool {
        // Check if git command is available
        if Command::new("git").arg("--version").output().is_err() {
            return false;
        }

        // Check if we're in a git repository
        path.join(".git").exists() || crate::vcs::find_vcs_root(path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestDir;
    use std::fs;
    use std::process::Command;

    fn setup_git_repo() -> TestDir {
        let temp_dir = TestDir::new().expect("should create temp dir");
        let path = temp_dir.path();

        // Initialize git repo and create commit in single Docker command to avoid race conditions
        fs::write(path.join("README.md"), "# Test Repo").expect("should write README");
        let init_script = [
            "git init -b main",
            "git config user.name 'Test User'",
            "git config user.email 'test@example.com'",
            "git add .",
            "git commit -m 'Initial commit'",
        ]
        .join(" && ");
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--entrypoint",
                "sh",
                "-v",
                &format!("{}:/workspace", path.display()),
                "-w",
                "/workspace",
                "alpine/git:latest",
                "-c",
                &init_script,
            ])
            .output()
            .expect("should execute docker command");
        assert!(
            output.status.success(),
            "Docker git setup failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        temp_dir
    }

    fn setup_git_repo_with_commit() -> TestDir {
        // Just return the basic repo since it already has a commit
        setup_git_repo()
    }

    fn setup_git_repo_with_tag(tag: &str) -> TestDir {
        let temp_dir = TestDir::new().expect("should create temp dir");
        let path = temp_dir.path();

        // Create repo, commit, and tag in single Docker command to avoid race conditions
        fs::write(path.join("README.md"), "# Test Repo").expect("should write README");
        let init_script = [
            "git init -b main",
            "git config user.name 'Test User'",
            "git config user.email 'test@example.com'",
            "git add .",
            "git commit -m 'Initial commit'",
            &format!("git tag {tag}"),
        ]
        .join(" && ");
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--entrypoint",
                "sh",
                "-v",
                &format!("{}:/workspace", path.display()),
                "-w",
                "/workspace",
                "alpine/git:latest",
                "-c",
                &init_script,
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Docker git setup with tag failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        temp_dir
    }

    #[test]
    #[ignore = "docker"]
    fn test_git_vcs_new() {
        let temp_dir = setup_git_repo();
        let result = GitVcs::new(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_vcs_new_no_repo() {
        let temp_dir = TestDir::new().expect("should create temp dir");
        let result = GitVcs::new(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "docker"]
    fn test_is_available() {
        let temp_dir = setup_git_repo();
        let git_vcs = GitVcs::new(temp_dir.path()).unwrap();
        assert!(git_vcs.is_available(temp_dir.path()));
    }

    #[test]
    fn test_is_available_no_repo() {
        let temp_dir = TestDir::new().expect("should create temp dir");
        let git_vcs = GitVcs {
            repo_path: temp_dir.path().to_path_buf(),
        };
        assert!(!git_vcs.is_available(temp_dir.path()));
    }

    #[test]
    #[ignore = "docker"]
    fn test_get_vcs_data_with_commit() {
        let temp_dir = setup_git_repo_with_commit();
        let git_vcs = GitVcs::new(temp_dir.path()).unwrap();
        let data = git_vcs.get_vcs_data().unwrap();

        assert!(!data.commit_hash.is_empty());
        assert!(!data.commit_hash_short.is_empty());
        assert!(data.commit_timestamp > 0);
        assert_eq!(data.tag_version, None);
        assert_eq!(data.distance, 0);
    }

    #[test]
    #[ignore = "docker"]
    fn test_get_vcs_data_with_tag() {
        let temp_dir = setup_git_repo_with_tag("v1.0.0");
        let git_vcs = GitVcs::new(temp_dir.path()).unwrap();
        let data = git_vcs.get_vcs_data().unwrap();

        assert!(!data.commit_hash.is_empty());
        assert!(!data.commit_hash_short.is_empty());
        assert!(data.commit_timestamp > 0);
        assert_eq!(data.tag_version, Some("v1.0.0".to_string()));
        assert_eq!(data.distance, 0);
    }

    #[test]
    #[ignore = "docker"]
    fn test_get_vcs_data_with_distance() {
        let temp_dir = setup_git_repo_with_tag("v1.0.0");
        let path = temp_dir.path();

        // Add another commit after tag using Docker
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--entrypoint",
                "sh",
                "-v",
                &format!("{}:/workspace", path.display()),
                "-w",
                "/workspace",
                "alpine/git:latest",
                "-c",
                "echo 'test content 2' > test2.txt && git add . && git commit -m 'second commit'",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Docker git second commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let git_vcs = GitVcs::new(temp_dir.path()).unwrap();
        let data = git_vcs.get_vcs_data().unwrap();

        assert_eq!(data.tag_version, Some("v1.0.0".to_string()));
        assert_eq!(data.distance, 1);
    }

    #[test]
    #[ignore = "docker"]
    fn test_dirty_working_directory() {
        let temp_dir = setup_git_repo_with_commit();
        let path = temp_dir.path();

        // Create untracked file
        fs::write(path.join("untracked.txt"), "untracked").unwrap();

        let git_vcs = GitVcs::new(temp_dir.path()).unwrap();
        let data = git_vcs.get_vcs_data().unwrap();

        assert!(data.is_dirty);
    }

    #[test]
    #[ignore = "docker"]
    fn test_clean_working_directory() {
        let temp_dir = setup_git_repo();
        let git_vcs = GitVcs::new(temp_dir.path()).unwrap();
        let data = git_vcs.get_vcs_data().unwrap();

        assert!(!data.is_dirty);
    }
}
