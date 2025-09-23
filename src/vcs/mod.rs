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

/// Version format for tag parsing
#[derive(Debug, Clone, PartialEq)]
pub enum VersionFormat {
    /// Try SemVer first, then PEP440
    Auto,
    /// Force SemVer parsing only
    SemVer,
    /// Force PEP440 parsing only
    Pep440,
    // TODO: Add custom regex support
    // Custom(String),
}

/// Detect and create appropriate VCS implementation
pub fn detect_vcs(path: &Path) -> Result<Box<dyn Vcs>> {
    let git_vcs = git::GitVcs::new(path)?;
    if git_vcs.is_available(path) {
        return Ok(Box::new(git_vcs));
    }

    Err(ZervError::VcsNotFound(
        "Not in a git repository (--source git)".to_string(),
    ))
}

/// Find the root directory of the VCS repository
pub fn find_vcs_root(start_path: &Path) -> Result<PathBuf> {
    let mut current = start_path.to_path_buf();

    loop {
        // Check for .git directory
        if current.join(".git").exists() {
            return Ok(current);
        }

        // Move up one directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
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
    fn test_version_format() {
        assert_eq!(VersionFormat::Auto, VersionFormat::Auto);
        assert_eq!(VersionFormat::SemVer, VersionFormat::SemVer);
        assert_eq!(VersionFormat::Pep440, VersionFormat::Pep440);
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
}
