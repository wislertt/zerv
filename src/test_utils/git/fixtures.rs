use super::GitOperations;
use crate::test_utils::{
    TestDir,
    get_git_impl,
};

/// High-level Git repository fixture for testing
pub struct GitRepoFixture {
    pub test_dir: TestDir,
    pub git_impl: Box<dyn GitOperations>,
}

impl GitRepoFixture {
    /// Create an empty repository without any tags
    pub fn empty() -> Result<Self, Box<dyn std::error::Error>> {
        let test_dir = TestDir::new()?;
        let git_impl = get_git_impl();

        // Perform atomic Git operations with error context
        git_impl
            .init_repo(&test_dir)
            .map_err(|e| format!("Failed to initialize Git repo: {e}"))?;

        // Verify repository was created properly
        if !test_dir.path().join(".git").exists() {
            return Err("Git repository was not properly initialized".into());
        }

        Ok(Self { test_dir, git_impl })
    }

    /// Create a repository with a clean tag (Tier 1: major.minor.patch)
    pub fn tagged(tag: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = Self::empty()?;

        fixture
            .git_impl
            .create_tag(&fixture.test_dir, tag)
            .map_err(|e| format!("Failed to create tag '{tag}': {e}"))?;

        Ok(fixture)
    }

    /// Create a repository with distance from tag (Tier 2: major.minor.patch.post<distance>+branch.<commit>)
    pub fn with_distance(tag: &str, commits: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = Self::tagged(tag)?;

        // Create additional commits for distance
        for i in 0..commits {
            fixture
                .test_dir
                .create_file(format!("file{}.txt", i + 1), "content")?;
            fixture
                .git_impl
                .create_commit(&fixture.test_dir, &format!("Commit {}", i + 1))?;
        }

        Ok(fixture)
    }

    /// Checkout to an existing or new branch
    pub fn checkout_branch(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl
            .create_branch(&self.test_dir, branch)
            .map_err(|e| format!("Failed to checkout branch '{}': {e}", branch))?;
        Ok(())
    }

    /// Make the working directory dirty with uncommitted changes
    pub fn make_dirty(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.test_dir
            .create_file("dirty_file.txt", "dirty content")?;
        Ok(())
    }

    /// Create a repository with dirty working directory (Tier 3: major.minor.patch.dev<timestamp>+branch.<commit>)
    pub fn dirty(tag: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = Self::tagged(tag)?;

        // Create uncommitted changes to make it dirty
        fixture
            .test_dir
            .create_file("dirty.txt", "uncommitted changes")?;

        Ok(fixture)
    }

    /// Get the path to the test directory
    pub fn path(&self) -> &std::path::Path {
        self.test_dir.path()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use serial_test::serial;

    use super::*;
    use crate::test_utils::should_run_docker_tests;

    #[test]
    #[serial(fixture_methods)]
    fn test_checkout_branch() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create fixture with tag");

        // Checkout a new branch
        fixture
            .checkout_branch("feature-test")
            .expect("Failed to checkout feature-test branch");

        // Verify branch was created
        let current_branch = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["branch", "--show-current"])
            .expect("Failed to get current branch");
        assert_eq!(current_branch.trim(), "feature-test");
    }

    #[test]
    #[serial(fixture_methods)]
    fn test_make_dirty() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create fixture with tag");

        // Make the working directory dirty
        fixture.make_dirty().expect("Failed to make fixture dirty");

        // Verify the dirty file exists
        assert!(fixture.path().join("dirty_file.txt").exists());

        // Verify git status shows uncommitted changes
        let status = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["status", "--porcelain"])
            .expect("Failed to get git status");
        assert!(status.contains("dirty_file.txt"));
    }

    static SHARED_V1_FIXTURE: Mutex<Option<(std::path::PathBuf, tempfile::TempDir)>> =
        Mutex::new(None);

    fn get_or_create_v1_fixture() -> std::path::PathBuf {
        let mut guard = SHARED_V1_FIXTURE.lock().unwrap();

        if let Some((path, _)) = guard.as_ref() {
            return path.clone();
        }

        let fixture =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create shared v1.0.0 fixture");

        let path = fixture.path().to_path_buf();
        let temp_dir = fixture.test_dir.into_inner();

        *guard = Some((path.clone(), temp_dir));
        path
    }

    #[test]
    #[serial(fixture_v1_shared)]
    fn test_tagged_fixture_creates_git_repo() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture_path = get_or_create_v1_fixture();

        // Should have Git repository
        assert!(fixture_path.exists());
        assert!(fixture_path.join(".git").exists());

        // Should have initial README.md from init_repo
        assert!(fixture_path.join("README.md").exists());
    }

    #[test]
    fn test_tagged_fixture_has_correct_tag() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v2.1.0").expect("Failed to create tagged fixture");

        // Verify tag exists in Git
        let output = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["tag", "-l"])
            .expect("Failed to list tags");
        assert!(output.contains("v2.1.0"), "Tag should exist: {output}");
    }

    #[test]
    fn test_with_distance_creates_commits() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::with_distance("v1.0.0", 3)
            .expect("Failed to create fixture with distance");

        // Should have Git repository
        assert!(fixture.path().exists());
        assert!(fixture.path().join(".git").exists());

        // Should have created additional files
        assert!(fixture.path().join("file1.txt").exists());
        assert!(fixture.path().join("file2.txt").exists());
        assert!(fixture.path().join("file3.txt").exists());

        // Verify Git log shows commits after tag
        let output = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["log", "--oneline"])
            .expect("Failed to get Git log");
        assert!(
            output.contains("Commit 1"),
            "Should have Commit 1: {output}"
        );
        assert!(
            output.contains("Commit 2"),
            "Should have Commit 2: {output}"
        );
        assert!(
            output.contains("Commit 3"),
            "Should have Commit 3: {output}"
        );
    }

    #[test]
    fn test_dirty_fixture_has_uncommitted_changes() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::dirty("v1.5.0").expect("Failed to create dirty fixture");

        // Should have Git repository
        assert!(fixture.path().exists());
        assert!(fixture.path().join(".git").exists());

        // Should have dirty file
        assert!(fixture.path().join("dirty.txt").exists());

        // Verify Git status shows uncommitted changes
        let output = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["status", "--porcelain"])
            .expect("Failed to get Git status");
        assert!(
            output.contains("dirty.txt"),
            "Should have uncommitted dirty.txt: {output}"
        );
    }

    #[test]
    fn test_fixture_path_access() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v0.1.0").expect("Failed to create fixture");

        // Path should be accessible and valid
        let path = fixture.path();
        assert!(path.exists());
        assert!(path.is_dir());

        // Should be able to read directory contents
        let entries: Vec<_> = std::fs::read_dir(path)
            .expect("Should be able to read directory")
            .collect();
        assert!(!entries.is_empty(), "Directory should not be empty");
    }

    #[test]
    fn test_multiple_fixtures_isolated() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture1 = GitRepoFixture::tagged("v1.0.0").expect("Failed to create fixture1");
        let fixture2 = GitRepoFixture::tagged("v2.0.0").expect("Failed to create fixture2");

        // Should have different paths
        assert_ne!(fixture1.path(), fixture2.path());

        // Both should exist independently
        assert!(fixture1.path().exists());
        assert!(fixture2.path().exists());

        // Should have different tags
        let tags1 = fixture1
            .git_impl
            .execute_git(&fixture1.test_dir, &["tag", "-l"])
            .expect("Failed to list tags1");
        let tags2 = fixture2
            .git_impl
            .execute_git(&fixture2.test_dir, &["tag", "-l"])
            .expect("Failed to list tags2");

        assert!(tags1.contains("v1.0.0"));
        assert!(tags2.contains("v2.0.0"));
    }

    #[test]
    #[serial(fixture_v1_shared)]
    fn test_zero_distance_commits() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture_path = get_or_create_v1_fixture();

        // Should still have Git repository and tag
        assert!(fixture_path.exists());
        assert!(fixture_path.join(".git").exists());

        let output = std::process::Command::new("git")
            .args(["tag", "-l"])
            .current_dir(&fixture_path)
            .output()
            .expect("Failed to run git command");
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(output_str.contains("v1.0.0"));

        // Should not have additional files (zero distance means no extra commits)
        assert!(!fixture_path.join("file1.txt").exists());
    }
}
