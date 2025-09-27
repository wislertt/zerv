use crate::error::{Result, ZervError};
use std::path::{Path, PathBuf};

pub mod git;

/// Version Control System trait for extracting repository metadata
pub trait Vcs {
    /// Extract VCS data from the repository
    fn get_vcs_data(&self) -> Result<VcsData>;

    /// Check if this VCS type is available in the given directory
    fn is_available(&self, path: &Path) -> bool;
}

/// VCS data extracted from repository
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VcsData {
    /// Latest version tag (e.g., "v1.2.3")
    pub tag_version: Option<String>,
    /// Distance from latest tag to HEAD
    pub distance: u32,
    /// Current commit hash (full)
    pub commit_hash: String,
    /// Current commit hash (short)
    pub commit_hash_short: String,
    /// Current branch name
    pub current_branch: Option<String>,
    /// Commit timestamp (Unix timestamp)
    pub commit_timestamp: i64,
    /// Tag timestamp (Unix timestamp)
    pub tag_timestamp: Option<i64>,
    /// Whether working directory is dirty
    pub is_dirty: bool,
}

/// Detect and create appropriate VCS implementation
pub fn detect_vcs(path: &Path) -> Result<Box<dyn Vcs>> {
    detect_vcs_with_limit(path, None)
}

/// Detect and create appropriate VCS implementation with optional depth limit
pub fn detect_vcs_with_limit(path: &Path, max_depth: Option<usize>) -> Result<Box<dyn Vcs>> {
    let git_vcs = git::GitVcs::new_with_limit(path, max_depth)?;
    if git_vcs.is_available(path) {
        return Ok(Box::new(git_vcs));
    }

    Err(ZervError::VcsNotFound(
        "Not in a git repository (--source git)".to_string(),
    ))
}

/// Find the root directory of the VCS repository
pub fn find_vcs_root(start_path: &Path) -> Result<PathBuf> {
    find_vcs_root_with_limit(start_path, None)
}

/// Find the root directory of the VCS repository with optional depth limit
pub fn find_vcs_root_with_limit(start_path: &Path, max_depth: Option<usize>) -> Result<PathBuf> {
    // Resolve the path to absolute to handle relative paths like ".." correctly
    let mut current = if start_path.is_absolute() {
        start_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(start_path)
    };

    let mut depth = 0;
    loop {
        // Check for .git directory
        if current.join(".git").exists() {
            return Ok(current);
        }

        // Check depth limit
        if let Some(max) = max_depth
            && depth >= max
        {
            break;
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => {
                current = parent.to_path_buf();
                depth += 1;
            }
            None => break,
        }
    }

    Err(ZervError::VcsNotFound(
        "Not in a git repository (--source git)".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_vcs_data_default() {
        let data = VcsData::default();
        assert_eq!(data.tag_version, None);
        assert_eq!(data.distance, 0);
        assert_eq!(data.commit_hash, "");
        assert_eq!(data.commit_hash_short, "");
        assert_eq!(data.current_branch, None);
        assert_eq!(data.commit_timestamp, 0);
        assert_eq!(data.tag_timestamp, None);
        assert!(!data.is_dirty);
    }

    #[test]
    fn test_find_vcs_root_no_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_vcs_root(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::VcsNotFound(_)));
    }

    #[test]
    fn test_find_vcs_root_error_message() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_vcs_root(temp_dir.path());

        match result {
            Err(ZervError::VcsNotFound(msg)) => {
                assert_eq!(msg, "Not in a git repository (--source git)");
            }
            _ => panic!("Expected VcsNotFound error with specific message"),
        }
    }

    #[test]
    fn test_detect_vcs_error_message() {
        let temp_dir = TempDir::new().unwrap();
        let result = detect_vcs(temp_dir.path());

        match result {
            Err(ZervError::VcsNotFound(msg)) => {
                assert_eq!(msg, "Not in a git repository (--source git)");
            }
            _ => panic!("Expected VcsNotFound error with specific message"),
        }
    }

    #[test]
    fn test_find_vcs_root_with_git() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let result = find_vcs_root(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_vcs_root_nested() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        let result = find_vcs_root(&nested_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[rstest]
    #[case(0, false)] // Depth 0 should fail
    #[case(1, false)] // Depth 1 should fail
    #[case(2, true)] // Depth 2 should succeed
    fn test_find_vcs_root_with_depth_limit(#[case] depth: usize, #[case] should_succeed: bool) {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        let result = find_vcs_root_with_limit(&nested_dir, Some(depth));

        if should_succeed {
            assert!(result.is_ok(), "Should succeed with depth {depth}");
            assert_eq!(result.unwrap(), temp_dir.path());
        } else {
            assert!(result.is_err(), "Should fail with depth {depth}");
            assert!(matches!(result, Err(ZervError::VcsNotFound(_))));
        }
    }

    #[test]
    fn test_find_vcs_root_with_no_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        // Test with no depth limit - should find the git repo
        let result = find_vcs_root_with_limit(&nested_dir, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_vcs_root_with_depth_limit_at_root() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        // Test with depth limit 0 at the git root - should find it
        let result = find_vcs_root_with_limit(temp_dir.path(), Some(0));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_find_vcs_root_relative_paths() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        // Create a subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Test with relative path ".." from subdir with depth limit 0 - should fail
        let result = find_vcs_root_with_limit(Path::new(".."), Some(0));
        // This should fail because we're not in a git repo when starting from current dir
        assert!(result.is_err());

        // Test with relative path "." with depth limit 0 - should work if we're in a git repo
        // But this test might be flaky depending on where it runs, so we'll test the logic differently
    }

    #[test]
    fn test_detect_vcs_with_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        // Test with depth limit 0 - should fail
        let result = detect_vcs_with_limit(&nested_dir, Some(0));
        assert!(result.is_err());
        assert!(matches!(result, Err(ZervError::VcsNotFound(_))));

        // Test with depth limit 2 - should succeed
        let result = detect_vcs_with_limit(&nested_dir, Some(2));
        assert!(result.is_ok());

        // Test with no depth limit - should succeed
        let result = detect_vcs_with_limit(&nested_dir, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_vcs_with_depth_limit_at_root() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        // Test with depth limit 0 at the git root - should succeed
        let result = detect_vcs_with_limit(temp_dir.path(), Some(0));
        assert!(result.is_ok());
    }
}
