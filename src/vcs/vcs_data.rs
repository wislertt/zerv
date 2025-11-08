#[derive(Debug, Clone, PartialEq, Default)]
pub struct VcsData {
    /// Latest version tag (e.g., "v1.2.3")
    pub tag_version: Option<String>,
    /// Distance from latest tag to HEAD
    pub distance: u32,
    /// Current commit hash (full)
    pub commit_hash: String,
    /// Commit hash prefix (e.g., "g" for Git following git describe convention)
    pub commit_hash_prefix: String,
    /// Current branch name
    pub current_branch: Option<String>,
    /// Commit timestamp (Unix timestamp)
    pub commit_timestamp: i64,
    /// Tag timestamp (Unix timestamp)
    pub tag_timestamp: Option<i64>,
    /// Whether the working directory is dirty (has uncommitted changes)
    pub is_dirty: bool,
    /// Whether the repository is shallow (limited history)
    pub is_shallow: bool,
}
