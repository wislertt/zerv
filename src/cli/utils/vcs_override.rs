use crate::cli::utils::format_handler::InputFormatHandler;
use crate::cli::version::VersionArgs;
use crate::error::{Result, ZervError};
use crate::vcs::VcsData;

/// Processor for applying VCS override options from CLI arguments
pub struct VcsOverrideProcessor;

impl VcsOverrideProcessor {
    /// Apply CLI overrides to VCS data
    /// Note: Early validation should be called before this method via args.validate()
    pub fn apply_overrides(mut vcs_data: VcsData, args: &VersionArgs) -> Result<VcsData> {
        // Apply --clean flag first (sets distance=0 and dirty=false)
        if args.clean {
            vcs_data.distance = 0;
            vcs_data.is_dirty = false;
        }

        // Apply individual overrides (these can override --clean if specified)
        if let Some(ref tag_version) = args.tag_version {
            // Parse the tag version with auto-detection for validation
            // Note: --input-format applies to stdin parsing, not tag-version overrides
            let _parsed_version = InputFormatHandler::parse_version_string(tag_version, "auto")?;

            // Store the original string in VcsData (validation ensures it's a valid format)
            vcs_data.tag_version = Some(tag_version.clone());
        }

        if let Some(distance) = args.distance {
            vcs_data.distance = distance;
        }

        // Apply dirty override using the helper method
        if let Some(dirty_value) = args.dirty_override() {
            vcs_data.is_dirty = dirty_value;
        }

        if let Some(ref current_branch) = args.current_branch {
            vcs_data.current_branch = Some(current_branch.clone());
        }

        if let Some(ref commit_hash) = args.commit_hash {
            vcs_data.commit_hash = commit_hash.clone();
            // Also update short hash (take first 7 characters)
            vcs_data.commit_hash_short = commit_hash.chars().take(7).collect();
        }

        // Apply additional overrides (these will be processed in ZervVars conversion)
        // Note: post, dev, pre_release_label, pre_release_num, epoch, custom are handled
        // in the ZervVars conversion phase, not in VcsData

        Ok(vcs_data)
    }

    /// Check if any VCS overrides are specified in the arguments
    pub fn has_overrides(args: &VersionArgs) -> bool {
        args.tag_version.is_some()
            || args.distance.is_some()
            || args.dirty
            || args.no_dirty
            || args.clean
            || args.current_branch.is_some()
            || args.commit_hash.is_some()
            || args.post.is_some()
            || args.dev.is_some()
            || args.pre_release_label.is_some()
            || args.pre_release_num.is_some()
            || args.epoch.is_some()
            || args.custom.is_some()
    }

    /// Apply context control logic (--bump-context vs --no-bump-context)
    pub fn apply_context_control(mut vcs_data: VcsData, args: &VersionArgs) -> Result<VcsData> {
        // Validate context flags
        if args.bump_context && args.no_bump_context {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --bump-context with --no-bump-context".to_string(),
            ));
        }

        // Apply context control
        if args.no_bump_context {
            // Force clean state - no VCS metadata
            vcs_data.distance = 0;
            vcs_data.is_dirty = false;
            vcs_data.current_branch = None;
            vcs_data.commit_hash = "unknown".to_string();
            vcs_data.commit_hash_short = "unknown".to_string();
        }
        // --bump-context is default behavior, no changes needed

        Ok(vcs_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vcs_data() -> VcsData {
        VcsData {
            tag_version: Some("v1.0.0".to_string()),
            distance: 5,
            commit_hash: "abcdef1234567890".to_string(),
            commit_hash_short: "abcdef1".to_string(),
            current_branch: Some("main".to_string()),
            commit_timestamp: 1234567890,
            tag_timestamp: Some(1234567800),
            is_dirty: true,
        }
    }

    fn create_test_args() -> VersionArgs {
        VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: None,
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: "semver".to_string(),
            tag_version: None,
            distance: None,
            dirty: false,
            no_dirty: false,
            clean: false,
            current_branch: None,
            commit_hash: None,
            post: None,
            dev: None,
            pre_release_label: None,
            pre_release_num: None,
            epoch: None,
            custom: None,
            bump_major: None,
            bump_minor: None,
            bump_patch: None,
            bump_distance: None,
            bump_post: None,
            bump_dev: None,
            bump_pre_release_num: None,
            bump_epoch: None,
            bump_context: false,
            no_bump_context: false,
            output_template: None,
            output_prefix: None,
            directory: None,
        }
    }

    #[test]
    fn test_apply_overrides_no_changes() {
        let vcs_data = create_test_vcs_data();
        let args = create_test_args();

        let result = VcsOverrideProcessor::apply_overrides(vcs_data.clone(), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vcs_data);
    }

    #[test]
    fn test_apply_overrides_tag_version() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.tag_version = Some("v2.0.0".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.tag_version, Some("v2.0.0".to_string()));
    }

    #[test]
    fn test_apply_overrides_tag_version_with_format() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.tag_version = Some("2.0.0".to_string());
        args.input_format = "semver".to_string();

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.tag_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_apply_overrides_tag_version_invalid_format() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.tag_version = Some("invalid-version".to_string());
        args.input_format = "semver".to_string();

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::InvalidVersion(_)));
    }

    #[test]
    fn test_apply_overrides_distance() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.distance = Some(10);

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.distance, 10);
    }

    #[test]
    fn test_apply_overrides_dirty_true() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.dirty = false;
        args.no_dirty = true;

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert!(!updated_data.is_dirty);
    }

    #[test]
    fn test_apply_overrides_dirty_false() {
        let mut vcs_data = create_test_vcs_data();
        vcs_data.is_dirty = false;
        let mut args = create_test_args();
        args.dirty = true;
        args.no_dirty = false;

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert!(updated_data.is_dirty);
    }

    #[test]
    fn test_apply_overrides_clean_flag() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.clean = true;

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.distance, 0);
        assert!(!updated_data.is_dirty);
    }

    #[test]
    fn test_apply_overrides_current_branch() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.current_branch = Some("feature/test".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(
            updated_data.current_branch,
            Some("feature/test".to_string())
        );
    }

    #[test]
    fn test_apply_overrides_commit_hash() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.commit_hash = Some("1234567890abcdef".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.commit_hash, "1234567890abcdef");
        assert_eq!(updated_data.commit_hash_short, "1234567");
    }

    #[test]
    fn test_apply_overrides_commit_hash_short() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.commit_hash = Some("abc".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.commit_hash, "abc");
        assert_eq!(updated_data.commit_hash_short, "abc");
    }

    #[test]
    fn test_apply_overrides_multiple() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.tag_version = Some("v3.0.0".to_string());
        args.distance = Some(15);
        args.dirty = false;
        args.no_dirty = true;
        args.current_branch = Some("develop".to_string());
        args.commit_hash = Some("fedcba0987654321".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated_data = result.unwrap();
        assert_eq!(updated_data.tag_version, Some("v3.0.0".to_string()));
        assert_eq!(updated_data.distance, 15);
        assert!(!updated_data.is_dirty);
        assert_eq!(updated_data.current_branch, Some("develop".to_string()));
        assert_eq!(updated_data.commit_hash, "fedcba0987654321");
        assert_eq!(updated_data.commit_hash_short, "fedcba0");
    }

    #[test]
    fn test_has_overrides_none() {
        let args = create_test_args();
        assert!(!VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_tag_version() {
        let mut args = create_test_args();
        args.tag_version = Some("v2.0.0".to_string());
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_distance() {
        let mut args = create_test_args();
        args.distance = Some(5);
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_dirty() {
        let mut args = create_test_args();
        args.dirty = true;
        args.no_dirty = false;
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_clean() {
        let mut args = create_test_args();
        args.clean = true;
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_current_branch() {
        let mut args = create_test_args();
        args.current_branch = Some("feature".to_string());
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_commit_hash() {
        let mut args = create_test_args();
        args.commit_hash = Some("abc123".to_string());
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_has_overrides_multiple() {
        let mut args = create_test_args();
        args.tag_version = Some("v2.0.0".to_string());
        args.distance = Some(5);
        assert!(VcsOverrideProcessor::has_overrides(&args));
    }

    #[test]
    fn test_clean_overrides_individual_options() {
        // Test that individual options can override --clean when specified after
        let _vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.clean = true;
        args.distance = Some(10); // This should override the clean distance=0

        // This should fail validation due to conflict
        let result = args.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_error_messages_quality() {
        // Test that error messages are clear and actionable
        let mut args = create_test_args();
        args.clean = true;
        args.distance = Some(5);

        let result = args.validate();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Conflicting options"));
        assert!(error_msg.contains("--clean"));
        assert!(error_msg.contains("--distance"));
        assert!(error_msg.contains("conflicting options"));
    }

    #[test]
    fn test_tag_version_parsing_with_different_formats() {
        let vcs_data = create_test_vcs_data();

        // Test SemVer format
        let mut args = create_test_args();
        args.tag_version = Some("2.0.0-alpha.1".to_string());
        args.input_format = "semver".to_string();

        let result = VcsOverrideProcessor::apply_overrides(vcs_data.clone(), &args);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().tag_version,
            Some("2.0.0-alpha.1".to_string())
        );

        // Test PEP440 format
        args.tag_version = Some("2.0.0a1".to_string());
        args.input_format = "pep440".to_string();

        let result = VcsOverrideProcessor::apply_overrides(vcs_data.clone(), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tag_version, Some("2.0.0a1".to_string()));

        // Test auto format
        args.tag_version = Some("2.0.0".to_string());
        args.input_format = "auto".to_string();

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tag_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_comprehensive_override_scenarios() {
        let vcs_data = create_test_vcs_data();

        // Scenario 1: Clean release simulation
        let mut args = create_test_args();
        args.clean = true;
        args.tag_version = Some("v2.0.0".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data.clone(), &args);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.tag_version, Some("v2.0.0".to_string()));
        assert_eq!(updated.distance, 0);
        assert!(!updated.is_dirty);

        // Scenario 2: Development build simulation
        let mut args = create_test_args();
        args.tag_version = Some("v1.9.0".to_string());
        args.distance = Some(25);
        args.dirty = true;
        args.no_dirty = false;
        args.current_branch = Some("feature/new-feature".to_string());

        let result = VcsOverrideProcessor::apply_overrides(vcs_data.clone(), &args);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.tag_version, Some("v1.9.0".to_string()));
        assert_eq!(updated.distance, 25);
        assert!(updated.is_dirty);
        assert_eq!(
            updated.current_branch,
            Some("feature/new-feature".to_string())
        );

        // Scenario 3: CI build simulation with specific commit
        let mut args = create_test_args();
        args.commit_hash = Some("1a2b3c4d5e6f7890".to_string());
        args.current_branch = Some("main".to_string());
        args.dirty = false;
        args.no_dirty = true;

        let result = VcsOverrideProcessor::apply_overrides(vcs_data, &args);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.commit_hash, "1a2b3c4d5e6f7890");
        assert_eq!(updated.commit_hash_short, "1a2b3c4");
        assert_eq!(updated.current_branch, Some("main".to_string()));
        assert!(!updated.is_dirty);
    }

    #[test]
    fn test_apply_context_control_no_bump_context() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.no_bump_context = true;

        let result = VcsOverrideProcessor::apply_context_control(vcs_data, &args);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.distance, 0);
        assert!(!updated.is_dirty);
        assert!(updated.current_branch.is_none());
        assert_eq!(updated.commit_hash, "unknown");
        assert_eq!(updated.commit_hash_short, "unknown");
    }

    #[test]
    fn test_apply_context_control_bump_context() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.bump_context = true;

        let result = VcsOverrideProcessor::apply_context_control(vcs_data.clone(), &args);
        assert!(result.is_ok());

        // Should be unchanged (default behavior)
        let updated = result.unwrap();
        assert_eq!(updated.distance, vcs_data.distance);
        assert_eq!(updated.is_dirty, vcs_data.is_dirty);
        assert_eq!(updated.current_branch, vcs_data.current_branch);
        assert_eq!(updated.commit_hash, vcs_data.commit_hash);
    }

    #[test]
    fn test_apply_context_control_conflicting_flags() {
        let vcs_data = create_test_vcs_data();
        let mut args = create_test_args();
        args.bump_context = true;
        args.no_bump_context = true;

        let result = VcsOverrideProcessor::apply_context_control(vcs_data, &args);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--bump-context"));
        assert!(error.to_string().contains("--no-bump-context"));
    }
}
