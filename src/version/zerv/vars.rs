// use crate::version::zerv::utils::normalize_pre_release_label;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json;

use crate::cli::utils::format_handler::InputFormatHandler;
use crate::cli::version::VersionArgs;
use crate::error::ZervError;
use crate::version::zerv::core::PreReleaseVar;

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
    fn derive_short_hash(hash: Option<&String>) -> Option<String> {
        hash.map(|h| {
            if h.len() >= 7 {
                h[..7].to_string()
            } else {
                h.clone()
            }
        })
    }

    pub fn get_bumped_commit_hash_short(&self) -> Option<String> {
        Self::derive_short_hash(self.bumped_commit_hash.as_ref())
    }

    pub fn get_last_commit_hash_short(&self) -> Option<String> {
        Self::derive_short_hash(self.last_commit_hash.as_ref())
    }

    /// Get custom value by key with dot-separated nested access
    /// Examples: "build_id", "metadata.author", "config.database.host"
    pub fn get_custom_value(&self, key: &str) -> Option<String> {
        let mut current = &self.custom;

        for part in key.split('.') {
            current = current.get(part)?;
        }

        match current {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => None, // Unsupported types (arrays, objects, null)
        }
    }

    /// Apply all CLI overrides to ZervVars including VCS and version components
    /// Note: Early validation should be called before this method via args.validate()
    pub fn apply_context_overrides(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply VCS-level overrides first
        self.apply_vcs_overrides(args)?;

        // Apply clean flag (overrides VCS settings if specified)
        self.apply_clean_flag(args)?;

        // Apply version-specific field overrides
        self.apply_tag_version_overrides(args)?;

        // Apply context control logic
        self.apply_context_control(args)?;

        Ok(())
    }

    /// Apply --clean flag (sets distance=None and dirty=false)
    fn apply_clean_flag(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        if args.overrides.clean {
            self.distance = None;
            self.dirty = Some(false);
        }
        Ok(())
    }

    /// Apply VCS-level overrides (distance, dirty, branch, commit_hash)
    fn apply_vcs_overrides(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply distance override
        if let Some(distance) = args.overrides.distance {
            self.distance = Some(distance as u64);
        }

        // Apply dirty override using the helper method
        if let Some(dirty_value) = args.dirty_override() {
            self.dirty = Some(dirty_value);
        }

        // Apply branch override
        if let Some(ref bumped_branch) = args.overrides.bumped_branch {
            self.bumped_branch = Some(bumped_branch.clone());
        }

        // Apply commit hash override
        if let Some(ref bumped_commit_hash) = args.overrides.bumped_commit_hash {
            self.bumped_commit_hash = Some(bumped_commit_hash.clone());
        }

        // Apply timestamp override
        if let Some(bumped_timestamp) = args.overrides.bumped_timestamp {
            self.bumped_timestamp = Some(bumped_timestamp as u64);
        }

        Ok(())
    }

    /// Apply version-specific field overrides
    fn apply_tag_version_overrides(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        // Apply tag version override (parse and extract components)
        if let Some(ref tag_version) = args.overrides.tag_version {
            // Use existing InputFormatHandler for parsing
            let version_object =
                InputFormatHandler::parse_version_string(tag_version, &args.main.input_format)?;
            let parsed_vars = ZervVars::from(version_object);

            // Apply parsed version components to self
            self.epoch = parsed_vars.epoch;
            self.major = parsed_vars.major;
            self.minor = parsed_vars.minor;
            self.patch = parsed_vars.patch;
            self.pre_release = parsed_vars.pre_release;
            self.post = parsed_vars.post;
            self.dev = parsed_vars.dev;
        }

        if let Some(ref custom_json) = args.overrides.custom {
            self.custom = serde_json::from_str(custom_json)
                .map_err(|e| ZervError::InvalidVersion(format!("Invalid custom JSON: {e}")))?;
        }

        Ok(())
    }

    /// Apply context control logic (--bump-context vs --no-bump-context)
    fn apply_context_control(&mut self, args: &VersionArgs) -> Result<(), ZervError> {
        if args.bumps.no_bump_context {
            // Force clean state - no VCS metadata
            self.distance = Some(0);
            self.dirty = Some(false);
            self.bumped_branch = None;
            self.bumped_commit_hash = None;
            self.bumped_timestamp = None;
        }
        // --bump-context is default behavior, no changes needed

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use rstest::rstest;

    use super::*;
    use crate::test_utils::VersionArgsFixture;
    use crate::version::zerv::core::PreReleaseLabel;

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

    // Tests for apply_overrides method covering all commented VcsData cases

    #[test]
    fn test_apply_overrides_clean_flag() {
        let mut vars = ZervVars {
            distance: Some(5),
            dirty: Some(true),
            ..Default::default()
        };

        let args = VersionArgsFixture::new().with_clean_flag(true).build();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.distance, None);
        assert_eq!(vars.dirty, Some(false));
    }

    #[test]
    fn test_apply_overrides_individual_vcs_overrides() {
        let mut vars = ZervVars {
            distance: Some(3),
            dirty: Some(false),
            bumped_branch: Some("main".to_string()),
            bumped_commit_hash: Some("abc123".to_string()),
            ..Default::default()
        };

        let args = VersionArgsFixture::new()
            .with_tag_version("v1.0.0")
            .with_distance(5)
            .with_dirty(true)
            .with_current_branch("feature/test")
            .with_commit_hash("abc123def")
            .build();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.distance, Some(5));
        assert_eq!(vars.dirty, Some(true));
        assert_eq!(vars.bumped_branch, Some("feature/test".to_string()));
        assert_eq!(vars.bumped_commit_hash, Some("abc123def".to_string())); // Full hash
    }

    #[test]
    fn test_apply_overrides_with_no_bump_context() {
        let mut vars = ZervVars {
            distance: Some(5),
            dirty: Some(true),
            bumped_branch: Some("main".to_string()),
            bumped_commit_hash: Some("abc123".to_string()),
            bumped_timestamp: Some(1234567890),
            ..Default::default()
        };

        let args = VersionArgs::try_parse_from(["version", "--no-bump-context"]).unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.distance, Some(0));
        assert_eq!(vars.dirty, Some(false));
        assert!(vars.bumped_branch.is_none());
        assert!(vars.bumped_commit_hash.is_none());
        assert!(vars.bumped_timestamp.is_none());
    }

    #[test]
    fn test_apply_overrides_with_bump_context() {
        let mut vars = ZervVars {
            distance: Some(5),
            dirty: Some(true),
            bumped_branch: Some("main".to_string()),
            bumped_commit_hash: Some("abc123".to_string()),
            ..Default::default()
        };

        let args = VersionArgs::try_parse_from(["version", "--bump-context"]).unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        // --bump-context is default behavior, so no changes should be made
        assert_eq!(vars.distance, Some(5));
        assert_eq!(vars.dirty, Some(true));
        assert_eq!(vars.bumped_branch, Some("main".to_string()));
        assert_eq!(vars.bumped_commit_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_apply_overrides_tag_version_only() {
        let mut vars = ZervVars::default();

        // Test that apply_overrides now only handles tag_version parsing and VCS overrides
        // Individual component overrides are handled in process_* methods
        let args = VersionArgsFixture::new().with_tag_version("2.0.0").build();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        // Tag version parsing should still work
        assert_eq!(vars.major, Some(2));
        assert_eq!(vars.minor, Some(0));
        assert_eq!(vars.patch, Some(0));
    }

    #[test]
    fn test_apply_overrides_individual_components_not_handled() {
        let mut vars = ZervVars::default();

        // Test that individual component overrides are NOT handled by apply_overrides
        let args = VersionArgsFixture::new()
            .with_post(10)
            .with_dev(5)
            .with_pre_release_label("alpha")
            .with_pre_release_num(1)
            .with_epoch(1)
            .build();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        // Individual component overrides should NOT be applied by apply_overrides
        assert_eq!(vars.post, None);
        assert_eq!(vars.dev, None);
        assert_eq!(vars.epoch, None);
        assert_eq!(vars.pre_release, None);
    }

    #[test]
    fn test_apply_overrides_custom_json() {
        let mut vars = ZervVars::default();

        let args = VersionArgs::try_parse_from([
            "zerv",
            "--custom",
            r#"{"build_id": 123, "env": "production"}"#,
        ])
        .unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.custom.get("build_id"), Some(&serde_json::json!(123)));
        assert_eq!(
            vars.custom.get("env"),
            Some(&serde_json::json!("production"))
        );
    }

    #[test]
    fn test_apply_overrides_tag_version_parsing() {
        let mut vars = ZervVars::default();

        let args = VersionArgs::try_parse_from([
            "zerv",
            "--tag-version",
            "v2.1.0-beta.1",
            "--input-format",
            "semver",
        ])
        .unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.major, Some(2));
        assert_eq!(vars.minor, Some(1));
        assert_eq!(vars.patch, Some(0));
        assert!(vars.pre_release.is_some());
        let pre_release = vars.pre_release.unwrap();
        assert_eq!(pre_release.label, PreReleaseLabel::Beta);
        assert_eq!(pre_release.number, Some(1));
    }

    #[test]
    fn test_apply_overrides_dirty_override_true() {
        let mut vars = ZervVars {
            dirty: Some(false),
            ..Default::default()
        };

        let args = VersionArgs::try_parse_from(["version", "--dirty"]).unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.dirty, Some(true));
    }

    #[test]
    fn test_apply_overrides_dirty_override_false() {
        let mut vars = ZervVars {
            dirty: Some(true),
            ..Default::default()
        };

        let args = VersionArgs::try_parse_from(["version", "--no-dirty"]).unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        assert_eq!(vars.dirty, Some(false));
    }

    #[test]
    fn test_apply_overrides_clean_overrides_other_vcs_settings() {
        let mut vars = ZervVars {
            distance: Some(10),
            dirty: Some(true),
            bumped_branch: Some("feature/test".to_string()),
            bumped_commit_hash: Some("def456".to_string()),
            ..Default::default()
        };

        let args = VersionArgs::try_parse_from([
            "zerv",
            "--clean",
            "--distance",
            "5",
            "--dirty",
            "--bumped-branch",
            "main",
            "--bumped-commit-hash",
            "abc123",
        ])
        .unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_ok());
        // Clean flag should override other VCS settings
        assert_eq!(vars.distance, None);
        assert_eq!(vars.dirty, Some(false));
        // But other overrides should still apply
        assert_eq!(vars.bumped_branch, Some("main".to_string()));
        assert_eq!(vars.bumped_commit_hash, Some("abc123".to_string())); // First 7 chars
    }

    #[test]
    fn test_apply_overrides_invalid_custom_json() {
        let mut vars = ZervVars::default();

        let args =
            VersionArgs::try_parse_from(["version", "--custom", r#"{"invalid": json}"#]).unwrap();
        let result = vars.apply_context_overrides(&args);

        assert!(result.is_err());
        match result {
            Err(ZervError::InvalidVersion(msg)) => {
                assert!(msg.contains("Invalid custom JSON"));
            }
            _ => panic!("Expected InvalidVersion error for invalid JSON"),
        }
    }

    #[test]
    fn test_get_custom_value() {
        let vars = ZervVars {
            custom: serde_json::json!({
                "build_id": "123",
                "environment": "prod",
                "enabled": true,
                "count": 42,
                "metadata": {
                    "author": "ci",
                    "timestamp": 1703123456,
                    "config": {
                        "database": {
                            "host": "localhost"
                        }
                    }
                },
                "array": [1, 2, 3],
                "null_value": null
            }),
            ..Default::default()
        };

        // Test simple string lookup
        assert_eq!(vars.get_custom_value("build_id"), Some("123".to_string()));
        assert_eq!(
            vars.get_custom_value("environment"),
            Some("prod".to_string())
        );

        // Test boolean lookup
        assert_eq!(vars.get_custom_value("enabled"), Some("true".to_string()));

        // Test number lookup
        assert_eq!(vars.get_custom_value("count"), Some("42".to_string()));

        // Test nested string lookup
        assert_eq!(
            vars.get_custom_value("metadata.author"),
            Some("ci".to_string())
        );

        // Test nested number lookup
        assert_eq!(
            vars.get_custom_value("metadata.timestamp"),
            Some("1703123456".to_string())
        );

        // Test deeply nested lookup
        assert_eq!(
            vars.get_custom_value("metadata.config.database.host"),
            Some("localhost".to_string())
        );

        // Test missing keys
        assert_eq!(vars.get_custom_value("nonexistent"), None);
        assert_eq!(vars.get_custom_value("metadata.nonexistent"), None);
        assert_eq!(vars.get_custom_value("metadata.config.nonexistent"), None);

        // Test unsupported types (arrays, objects, null)
        assert_eq!(vars.get_custom_value("array"), None);
        assert_eq!(vars.get_custom_value("metadata"), None);
        assert_eq!(vars.get_custom_value("null_value"), None);

        // Test empty key
        assert_eq!(vars.get_custom_value(""), None);
    }
}
