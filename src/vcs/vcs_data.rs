use crate::cli::utils::format_handler::InputFormatHandler;
use crate::cli::version::VersionArgs;
use crate::error::{Result, ZervError};

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
    /// Whether the working directory is dirty (has uncommitted changes)
    pub is_dirty: bool,
    /// Whether the repository is shallow (limited history)
    pub is_shallow: bool,
}

impl VcsData {
    /// Apply all CLI overrides to VCS data including context control
    /// Note: Early validation should be called before this method via args.validate()
    pub fn apply_overrides(mut self, args: &VersionArgs) -> Result<Self> {
        // Apply clean flag first
        self = self.apply_clean_flag(args)?;

        // Apply individual field overrides
        self = self.apply_field_overrides(args)?;

        // Apply context control
        self = self.apply_context_control(args)?;

        Ok(self)
    }

    /// Apply --clean flag (sets distance=0 and dirty=false)
    fn apply_clean_flag(mut self, args: &VersionArgs) -> Result<Self> {
        if args.clean {
            self.distance = 0;
            self.is_dirty = false;
        }
        Ok(self)
    }

    /// Apply individual field overrides from CLI arguments
    fn apply_field_overrides(mut self, args: &VersionArgs) -> Result<Self> {
        // Apply tag version override with validation
        if let Some(ref tag_version) = args.tag_version {
            // Parse the tag version with auto-detection for validation
            // Note: --input-format applies to stdin parsing, not tag-version overrides
            let _parsed_version = InputFormatHandler::parse_version_string(tag_version, "auto")?;
            self.tag_version = Some(tag_version.clone());
        }

        // Apply distance override
        if let Some(distance) = args.distance {
            self.distance = distance;
        }

        // Apply dirty override using the helper method
        if let Some(dirty_value) = args.dirty_override() {
            self.is_dirty = dirty_value;
        }

        // Apply branch override
        if let Some(ref current_branch) = args.current_branch {
            self.current_branch = Some(current_branch.clone());
        }

        // Apply commit hash override
        if let Some(ref commit_hash) = args.commit_hash {
            self.commit_hash = commit_hash.clone();
            // Also update short hash (take first 7 characters)
            self.commit_hash_short = commit_hash.chars().take(7).collect();
        }

        // Note: post, dev, pre_release_label, pre_release_num, epoch, custom are handled
        // in the ZervVars conversion phase, not in VcsData

        Ok(self)
    }

    /// Apply context control logic (--bump-context vs --no-bump-context)
    fn apply_context_control(mut self, args: &VersionArgs) -> Result<Self> {
        // Validate context flags
        if args.bump_context && args.no_bump_context {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --bump-context with --no-bump-context".to_string(),
            ));
        }

        // Apply context control
        if args.no_bump_context {
            // Force clean state - no VCS metadata
            self.distance = 0;
            self.is_dirty = false;
            self.current_branch = None;
            self.commit_hash = "unknown".to_string();
            self.commit_hash_short = "unknown".to_string();
        }
        // --bump-context is default behavior, no changes needed

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::VersionArgsFixture;
    use clap::Parser;

    #[test]
    fn test_apply_overrides_clean_flag() {
        let vcs_data = VcsData {
            distance: 5,
            is_dirty: true,
            ..Default::default()
        };

        let args = VersionArgsFixture::with_clean();
        let result = vcs_data.apply_overrides(&args);

        assert!(result.is_ok());
        let updated_vcs_data = result.unwrap();
        assert_eq!(updated_vcs_data.distance, 0);
        assert!(!updated_vcs_data.is_dirty);
    }

    #[test]
    fn test_apply_overrides_individual_overrides() {
        let vcs_data = VcsData {
            tag_version: Some("v1.0.0".to_string()),
            distance: 3,
            is_dirty: false,
            current_branch: Some("main".to_string()),
            commit_hash: "abc123".to_string(),
            commit_hash_short: "abc123".to_string(),
            ..Default::default()
        };

        let args = VersionArgsFixture::with_overrides();
        let result = vcs_data.apply_overrides(&args);

        assert!(result.is_ok());
        let updated_vcs_data = result.unwrap();
        assert_eq!(updated_vcs_data.tag_version, Some("v2.0.0".to_string()));
        assert_eq!(updated_vcs_data.distance, 5);
        assert!(updated_vcs_data.is_dirty);
        assert_eq!(
            updated_vcs_data.current_branch,
            Some("feature/test".to_string())
        );
        assert_eq!(updated_vcs_data.commit_hash, "abc123def456");
        assert_eq!(updated_vcs_data.commit_hash_short, "abc123d");
    }

    #[test]
    fn test_apply_overrides_with_no_bump_context() {
        let vcs_data = VcsData {
            distance: 5,
            is_dirty: true,
            current_branch: Some("main".to_string()),
            commit_hash: "abc123".to_string(),
            commit_hash_short: "abc123".to_string(),
            ..Default::default()
        };

        let args = VersionArgs::try_parse_from(["zerv", "--no-bump-context"]).unwrap();
        let result = vcs_data.apply_overrides(&args);

        assert!(result.is_ok());
        let updated_vcs_data = result.unwrap();
        assert_eq!(updated_vcs_data.distance, 0);
        assert!(!updated_vcs_data.is_dirty);
        assert!(updated_vcs_data.current_branch.is_none());
        assert_eq!(updated_vcs_data.commit_hash, "unknown");
        assert_eq!(updated_vcs_data.commit_hash_short, "unknown");
    }

    #[test]
    fn test_apply_overrides_with_conflicting_context_flags() {
        let vcs_data = VcsData::default();
        let args =
            VersionArgs::try_parse_from(["zerv", "--bump-context", "--no-bump-context"]).unwrap();
        let result = vcs_data.apply_overrides(&args);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--bump-context"));
        assert!(error.to_string().contains("--no-bump-context"));
    }
}
