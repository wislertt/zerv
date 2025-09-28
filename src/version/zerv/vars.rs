use crate::version::zerv::core::PreReleaseVar;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ZervVars {
    // Core version fields
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,
    pub pre_release: Option<PreReleaseVar>,
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // VCS state fields
    pub distance: Option<u64>,
    pub dirty: Option<bool>,

    // Bumped fields (for template access)
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields (for template access)
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables
    #[serde(default = "default_custom_value")]
    pub custom: serde_json::Value,
}

/// Default value for custom field - returns an empty JSON object
fn default_custom_value() -> serde_json::Value {
    serde_json::json!({})
}

impl ZervVars {
    /// Get the short commit hash, derived from bumped_commit_hash
    /// Returns the first 7 characters of the full commit hash, or None if not available
    pub fn get_bumped_commit_hash_short(&self) -> Option<String> {
        Self::derive_short_hash(self.bumped_commit_hash.as_ref())
    }

    /// Get the short commit hash for the last commit, derived from last_commit_hash
    /// Returns the first 7 characters of the full commit hash, or None if not available
    pub fn get_last_commit_hash_short(&self) -> Option<String> {
        Self::derive_short_hash(self.last_commit_hash.as_ref())
    }

    /// Derive short hash from full hash (7 characters or less if hash is shorter)
    fn derive_short_hash(hash: Option<&String>) -> Option<String> {
        hash.map(|h| {
            if h.len() >= 7 {
                h[..7].to_string()
            } else {
                h.clone()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Some("abcdef1234567890"), Some("abcdef1"))]
    #[case(Some("abc123"), Some("abc123"))]
    #[case(Some("a"), Some("a"))]
    #[case(Some(""), Some(""))]
    #[case(None, None)]
    fn test_commit_hash_short_derivation(
        #[case] input: Option<&str>,
        #[case] expected: Option<&str>,
    ) {
        let vars = ZervVars {
            bumped_commit_hash: input.map(|s| s.to_string()),
            ..Default::default()
        };

        assert_eq!(
            vars.get_bumped_commit_hash_short(),
            expected.map(|s| s.to_string())
        );
    }

    #[rstest]
    #[case(Some("fedcba0987654321"), Some("fedcba0"))]
    #[case(Some("def456"), Some("def456"))]
    #[case(Some("x"), Some("x"))]
    #[case(Some(""), Some(""))]
    #[case(None, None)]
    fn test_last_commit_hash_short_derivation(
        #[case] input: Option<&str>,
        #[case] expected: Option<&str>,
    ) {
        let vars = ZervVars {
            last_commit_hash: input.map(|s| s.to_string()),
            ..Default::default()
        };

        assert_eq!(
            vars.get_last_commit_hash_short(),
            expected.map(|s| s.to_string())
        );
    }

    #[test]
    fn test_custom_variables() {
        let mut vars = ZervVars::default();
        vars.custom["build_id"] = serde_json::json!(123);
        vars.custom["env"] = serde_json::json!("production");

        assert_eq!(vars.custom.get("build_id"), Some(&serde_json::json!(123)));
        assert_eq!(
            vars.custom.get("env"),
            Some(&serde_json::json!("production"))
        );
        assert_eq!(vars.custom.get("nonexistent"), None);
    }

    #[test]
    fn test_serialization() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            bumped_commit_hash: Some("abcdef1234567890".to_string()),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&vars).unwrap();
        let parsed: ZervVars = serde_json::from_str(&serialized).unwrap();

        assert_eq!(parsed.major, Some(1));
        assert_eq!(
            parsed.bumped_commit_hash,
            Some("abcdef1234567890".to_string())
        );
        assert_eq!(
            parsed.get_bumped_commit_hash_short(),
            Some("abcdef1".to_string())
        );
    }
}
