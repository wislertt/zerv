use super::GitOperations;
use crate::test_utils::{TestDir, get_git_impl};

/// High-level Git repository fixture for testing
pub struct GitRepoFixture {
    pub test_dir: TestDir,
    pub git_impl: Box<dyn GitOperations>,
}

impl GitRepoFixture {
    /// Create a repository with a clean tag (Tier 1: major.minor.patch)
    pub fn tagged(tag: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let test_dir = TestDir::new()?;
        let git_impl = get_git_impl();

        git_impl.init_repo(&test_dir)?;
        git_impl.create_tag(&test_dir, tag)?;

        Ok(Self { test_dir, git_impl })
    }

    /// Create a repository with distance from tag (Tier 2: major.minor.patch.post<distance>+branch.<commit>)
    pub fn with_distance(tag: &str, commits: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let test_dir = TestDir::new()?;
        let git_impl = get_git_impl();

        git_impl.init_repo(&test_dir)?;
        git_impl.create_tag(&test_dir, tag)?;

        // Create additional commits for distance
        for i in 0..commits {
            test_dir.create_file(format!("file{}.txt", i + 1), "content")?;
            git_impl.create_commit(&test_dir, &format!("Commit {}", i + 1))?;
        }

        Ok(Self { test_dir, git_impl })
    }

    /// Create a repository with dirty working directory (Tier 3: major.minor.patch.dev<timestamp>+branch.<commit>)
    pub fn dirty(tag: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let test_dir = TestDir::new()?;
        let git_impl = get_git_impl();

        git_impl.init_repo(&test_dir)?;
        git_impl.create_tag(&test_dir, tag)?;

        // Create uncommitted changes to make it dirty
        test_dir.create_file("dirty.txt", "uncommitted changes")?;

        Ok(Self { test_dir, git_impl })
    }

    /// Get the path to the test directory
    pub fn path(&self) -> &std::path::Path {
        self.test_dir.path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::should_run_docker_tests;

    #[test]
    fn test_tagged_fixture_creates_git_repo() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged fixture");

        // Should have Git repository
        assert!(fixture.path().exists());
        assert!(fixture.path().join(".git").exists());

        // Should have initial README.md from init_repo
        assert!(fixture.path().join("README.md").exists());
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
    fn test_zero_distance_commits() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::with_distance("v1.0.0", 0)
            .expect("Failed to create fixture with zero distance");

        // Should still have Git repository and tag
        assert!(fixture.path().exists());
        assert!(fixture.path().join(".git").exists());

        let output = fixture
            .git_impl
            .execute_git(&fixture.test_dir, &["tag", "-l"])
            .expect("Failed to list tags");
        assert!(output.contains("v1.0.0"));

        // Should not have additional files
        assert!(!fixture.path().join("file1.txt").exists());
    }
}
