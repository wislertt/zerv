use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

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

    /// Create a repository with a clean annotated tag (Tier 1: major.minor.patch)
    pub fn tagged_annotated(tag: &str, message: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let fixture = Self::empty()?;

        fixture
            .git_impl
            .create_annotated_tag(&fixture.test_dir, tag, message)
            .map_err(|e| format!("Failed to create annotated tag '{tag}': {e}"))?;

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

    /// Create a new branch without checking it out
    pub fn create_branch(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl
            .create_branch(&self.test_dir, branch)
            .map_err(|e| format!("Failed to create branch '{}': {e}", branch))?;
        Ok(())
    }

    /// Checkout to an existing branch
    pub fn checkout_branch(&self, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.git_impl
            .checkout_branch(&self.test_dir, branch)
            .map_err(|e| format!("Failed to checkout branch '{}': {e}", branch))?;
        Ok(())
    }

    /// Builder-style: Create a new branch without checking it out
    pub fn with_branch(self, branch: &str) -> Self {
        self.git_impl
            .create_branch(&self.test_dir, branch)
            .unwrap_or_else(|e| panic!("Failed to create branch '{}': {}", branch, e));
        self
    }

    /// Builder-style: Checkout to an existing branch
    pub fn with_checkout(self, branch: &str) -> Self {
        self.git_impl
            .checkout_branch(&self.test_dir, branch)
            .unwrap_or_else(|e| panic!("Failed to checkout branch '{}': {}", branch, e));
        self
    }

    /// Make the working directory dirty with uncommitted changes
    pub fn make_dirty(&self) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let content = format!("dirty content {}", timestamp);
        self.test_dir.create_file("dirty_file.txt", &content)?;
        Ok(())
    }

    /// Builder-style: Make the working directory dirty with uncommitted changes
    pub fn with_dirty(self) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let content = format!("dirty content {}", timestamp);
        self.test_dir
            .create_file("dirty_file.txt", &content)
            .expect("Failed to create dirty file");
        self
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

    /// Create a single tag
    pub fn create_tag(self, tag: &str) -> Self {
        self.git_impl
            .create_tag(&self.test_dir, tag)
            .unwrap_or_else(|e| panic!("Failed to create tag '{}': {}", tag, e));
        self
    }

    /// Create a single annotated tag
    pub fn create_annotated_tag(self, tag: &str, message: &str) -> Self {
        self.git_impl
            .create_annotated_tag(&self.test_dir, tag, message)
            .unwrap_or_else(|e| panic!("Failed to create annotated tag '{}': {}", tag, e));
        self
    }

    /// Builder-style: Create a single annotated tag
    pub fn with_annotated_tag(self, tag: &str, message: &str) -> Self {
        self.create_annotated_tag(tag, message)
    }

    /// Create a commit without tagging (for building distance)
    pub fn commit(self, message: &str) -> Self {
        // Create a file change to ensure there's something to commit
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        self.test_dir
            .create_file(
                format!("commit_{}.txt", timestamp),
                &format!("Content for {}", message),
            )
            .expect("Failed to create commit file");

        self.git_impl
            .create_commit(&self.test_dir, message)
            .unwrap_or_else(|e| panic!("Failed to create commit '{}': {}", message, e));
        self
    }

    /// Get current HEAD commit hash
    pub fn get_head_commit(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = self
            .git_impl
            .execute_git(&self.test_dir, &["rev-parse", "HEAD"])
            .map_err(|e| format!("Failed to get HEAD commit: {e}"))?;
        Ok(output.trim().to_string())
    }

    /// Checkout to a specific commit (preserves commit history)
    pub fn checkout(self, commit: &str) -> Self {
        self.git_impl
            .execute_git(&self.test_dir, &["checkout", commit])
            .unwrap_or_else(|e| panic!("Failed to checkout commit '{}': {}", commit, e));
        self
    }

    /// Merge a branch into the current branch
    pub fn merge_branch(self, branch: &str) -> Self {
        self.git_impl
            .merge_branch(&self.test_dir, branch)
            .unwrap_or_else(|e| panic!("Failed to merge branch '{}': {}", branch, e));
        self
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

        // Create and checkout a new branch
        fixture
            .create_branch("feature-test")
            .expect("Failed to create feature-test branch");
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

    #[test]
    #[serial(fixture_methods)]
    fn test_merge_branch_and_checkout() {
        if !should_run_docker_tests() {
            return;
        }

        let mut fixture =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create fixture with tag");

        // Test 1: Branch creation and merging
        fixture
            .create_branch("feature-branch")
            .expect("Failed to create feature branch");
        fixture
            .checkout_branch("feature-branch")
            .expect("Failed to checkout feature branch");

        // Add a commit on the feature branch
        fixture = fixture.commit("Feature commit");

        // Switch back to main and merge
        fixture
            .checkout_branch("main")
            .expect("Failed to checkout main");
        fixture = fixture.merge_branch("feature-branch");

        // Verify merge was successful by checking the commit exists
        let output = fixture
            .git_impl
            .execute_git(
                &fixture.test_dir,
                &["log", "--oneline", "--grep=Feature commit"],
            )
            .expect("Failed to check git log");
        assert!(
            output.contains("Feature commit"),
            "Feature commit should be merged into main"
        );

        // Test 2: Checkout preserves commit history (safer than reset)
        // Create more commits with tags
        fixture = fixture
            .commit("Feature A")
            .create_tag("v2.0.0")
            .commit("Feature B")
            .create_tag("v3.0.0");

        // Get v2.0.0 commit hash
        let v2_commit = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["rev-list", "-n", "1", "v2.0.0"])
            .expect("Failed to get v2.0.0 commit");
        let v2_commit = v2_commit.trim();

        // Checkout to v2.0.0 (preserves history)
        fixture = fixture.checkout(v2_commit);

        // Verify we can still see v3.0.0 tag even though we're checked out to v2.0.0
        let all_tags = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["tag", "--list"])
            .expect("Failed to list tags");
        assert!(
            all_tags.contains("v3.0.0"),
            "v3.0.0 tag should still exist after checkout"
        );
    }
}
