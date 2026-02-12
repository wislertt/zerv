#[derive(Debug, Clone, PartialEq, Default)]
pub struct VcsData {
    /// Latest version tag (e.g., "v1.2.3")
    pub tag_version: Option<String>,
    pub tag_commit_hash: Option<String>,
    pub tag_timestamp: Option<i64>,

    pub commit_hash: String,
    pub commit_hash_prefix: String,
    pub commit_timestamp: i64,
    pub current_branch: Option<String>,
    pub is_dirty: bool,
    pub distance: u32,
}
