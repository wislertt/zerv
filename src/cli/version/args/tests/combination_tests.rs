use clap::Parser;

use super::super::*;
use crate::test_utils::VersionArgsFixture;
use crate::utils::constants::{
    formats,
    sources,
};

#[test]
fn test_version_args_defaults() {
    let args = VersionArgs::try_parse_from(["version"]).unwrap();
    assert_eq!(args.main.source, sources::GIT);
    assert!(args.main.schema.is_none());
    assert!(args.main.schema_ron.is_none());
    assert_eq!(args.main.input_format, formats::AUTO);
    assert_eq!(args.main.output_format, formats::SEMVER);

    // VCS override options should be None/false by default
    assert!(args.overrides.tag_version.is_none());
    assert!(args.overrides.distance.is_none());
    assert!(!args.overrides.dirty);
    assert!(!args.overrides.no_dirty);
    assert!(!args.overrides.clean);
    assert!(args.overrides.current_branch.is_none());
    assert!(args.overrides.commit_hash.is_none());
    assert!(args.overrides.post.is_none());
    assert!(args.overrides.dev.is_none());
    assert!(args.overrides.pre_release_label.is_none());
    assert!(args.overrides.pre_release_num.is_none());
    assert!(args.overrides.epoch.is_none());
    assert!(args.overrides.custom.is_none());

    // Bump options should be None by default
    assert!(args.bumps.bump_major.is_none());
    assert!(args.bumps.bump_minor.is_none());
    assert!(args.bumps.bump_patch.is_none());
    assert!(args.bumps.bump_post.is_none());
    assert!(args.bumps.bump_dev.is_none());
    assert!(args.bumps.bump_pre_release_num.is_none());
    assert!(args.bumps.bump_epoch.is_none());
    assert!(args.bumps.bump_pre_release_label.is_none());

    // Schema-based bump options should be empty by default
    assert!(args.bumps.bump_core.is_empty());
    assert!(args.bumps.bump_extra_core.is_empty());
    assert!(args.bumps.bump_build.is_empty());

    // Context control options should be false by default
    assert!(!args.bumps.bump_context);
    assert!(!args.bumps.no_bump_context);

    // Output options should be None by default
    assert!(args.main.output_template.is_none());
    assert!(args.main.output_prefix.is_none());
}

#[test]
fn test_version_args_with_overrides() {
    let args = VersionArgs::try_parse_from([
        "zerv",
        "--tag-version",
        "v2.0.0",
        "--distance",
        "5",
        "--dirty",
        "--current-branch",
        "feature/test",
        "--commit-hash",
        "abc123",
        "--input-format",
        "semver",
        "--output-prefix",
        "version:",
    ])
    .unwrap();

    assert_eq!(args.overrides.tag_version, Some("v2.0.0".to_string()));
    assert_eq!(args.overrides.distance, Some(5));
    assert!(args.overrides.dirty);
    assert!(!args.overrides.no_dirty);
    assert!(!args.overrides.clean);
    assert_eq!(
        args.overrides.current_branch,
        Some("feature/test".to_string())
    );
    assert_eq!(args.overrides.commit_hash, Some("abc123".to_string()));
    assert_eq!(args.main.input_format, formats::SEMVER);
    assert_eq!(args.main.output_prefix, Some("version:".to_string()));
}

#[test]
fn test_version_args_clean_flag() {
    let args = VersionArgs::try_parse_from(["version", "--clean"]).unwrap();

    assert!(args.overrides.clean);
    assert!(args.overrides.distance.is_none());
    assert!(!args.overrides.dirty);
    assert!(!args.overrides.no_dirty);
}

#[test]
fn test_version_args_dirty_flags() {
    // Test --dirty flag
    let args = VersionArgs::try_parse_from(["version", "--dirty"]).unwrap();
    assert!(args.overrides.dirty);
    assert!(!args.overrides.no_dirty);

    // Test --no-dirty flag
    let args = VersionArgs::try_parse_from(["version", "--no-dirty"]).unwrap();
    assert!(!args.overrides.dirty);
    assert!(args.overrides.no_dirty);

    // Test both flags together should fail early validation
    let mut args = VersionArgs::try_parse_from(["version", "--dirty", "--no-dirty"]).unwrap();
    assert!(args.overrides.dirty);
    assert!(args.overrides.no_dirty);

    // The conflict should be caught by early validation
    let result = args.validate();
    assert!(result.is_err());
}

#[test]
fn test_dirty_override_helper() {
    // Test --dirty flag
    let args = VersionArgs::try_parse_from(["version", "--dirty"]).unwrap();
    assert_eq!(args.dirty_override(), Some(true));

    // Test --no-dirty flag
    let args = VersionArgs::try_parse_from(["version", "--no-dirty"]).unwrap();
    assert_eq!(args.dirty_override(), Some(false));

    // Test neither flag (use VCS)
    let args = VersionArgs::try_parse_from(["version"]).unwrap();
    assert_eq!(args.dirty_override(), None);
}

#[rstest]
#[case(&["version"])]
#[case(&["version", "--dirty"])]
#[case(&["version", "--no-dirty"])]
#[case(&["version", "--clean"])]
#[case(&["version", "--distance", "5"])]
fn test_validate_no_conflicts(#[case] args: &[&str]) {
    let mut args = VersionArgs::try_parse_from(args).unwrap();
    assert!(args.validate().is_ok());
}

use rstest::rstest;

#[rstest]
#[case(&["version", "--dirty", "--no-dirty"], &["--dirty", "--no-dirty"])]
#[case(&["version", "--clean", "--distance", "5"], &["--clean", "--distance"])]
#[case(&["version", "--clean", "--dirty"], &["--clean", "--dirty"])]
#[case(&["version", "--clean", "--no-dirty"], &["--clean", "--no-dirty"])]
#[case(&["version", "--bump-context", "--no-bump-context"], &["--bump-context", "--no-bump-context"])]
#[case(&["zerv", "--no-bump-context", "--dirty"], &["--no-bump-context", "--dirty"])]
fn test_validate_conflicting_options(#[case] args: &[&str], #[case] expected_flags: &[&str]) {
    let mut args = VersionArgs::try_parse_from(args).unwrap();
    let result = args.validate();
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(
        error,
        crate::error::ZervError::ConflictingOptions(_)
    ));

    let error_msg = error.to_string();
    assert!(error_msg.contains("conflicting options"));
    for flag in expected_flags {
        assert!(error_msg.contains(flag));
    }
}

#[test]
fn test_validate_clean_with_non_conflicting_options() {
    let mut args = VersionArgs::try_parse_from([
        "zerv",
        "--clean",
        "--tag-version",
        "v2.0.0",
        "--current-branch",
        "main",
        "--commit-hash",
        "abc123",
    ])
    .unwrap();
    assert!(args.validate().is_ok());
}

#[test]
fn test_validate_multiple_conflicts() {
    // Test that validation fails on the first conflict found
    let mut args = VersionArgs::try_parse_from([
        "zerv",
        "--clean",
        "--distance",
        "5",
        "--dirty",
        "--no-dirty",
    ])
    .unwrap();
    let result = args.validate();
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    // Should fail on the first conflict (dirty flags conflict comes first)
    assert!(error_msg.contains("--dirty"));
    assert!(error_msg.contains("--no-dirty"));
    assert!(error_msg.contains("conflicting options"));
}

#[test]
fn test_validate_error_message_quality() {
    // Test that error messages are clear and actionable
    let mut args = VersionArgs::try_parse_from(["version", "--dirty", "--no-dirty"]).unwrap();
    let result = args.validate();
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Conflicting options"));
    assert!(error_msg.contains("--dirty"));
    assert!(error_msg.contains("--no-dirty"));
    assert!(error_msg.contains("conflicting options"));
    assert!(error_msg.contains("Cannot use"));
}

#[test]
fn test_context_control_all_scenarios() {
    // Test all 4 possible states of (bump_context, no_bump_context)

    // Scenario 1: (false, false) - Neither flag provided: should default to bump-context
    let mut args = VersionArgs::try_parse_from(["version"]).unwrap();
    assert!(!args.bumps.bump_context);
    assert!(!args.bumps.no_bump_context);
    assert!(args.validate().is_ok());
    assert!(args.bumps.bump_context);
    assert!(!args.bumps.no_bump_context);

    // Scenario 2: (true, false) - Explicit --bump-context: should remain unchanged
    let mut args = VersionArgs::try_parse_from(["version", "--bump-context"]).unwrap();
    assert!(args.bumps.bump_context);
    assert!(!args.bumps.no_bump_context);
    assert!(args.validate().is_ok());
    assert!(args.bumps.bump_context);
    assert!(!args.bumps.no_bump_context);

    // Scenario 3: (false, true) - Explicit --no-bump-context: should remain unchanged
    let mut args = VersionArgs::try_parse_from(["version", "--no-bump-context"]).unwrap();
    assert!(!args.bumps.bump_context);
    assert!(args.bumps.no_bump_context);
    assert!(args.validate().is_ok());
    assert!(!args.bumps.bump_context);
    assert!(args.bumps.no_bump_context);

    // Scenario 4: (true, true) - Both flags provided: should return error
    let mut args =
        VersionArgs::try_parse_from(["version", "--bump-context", "--no-bump-context"]).unwrap();
    assert!(args.bumps.bump_context);
    assert!(args.bumps.no_bump_context);
    let result = args.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error,
        crate::error::ZervError::ConflictingOptions(_)
    ));
    assert!(error.to_string().contains("--bump-context"));
    assert!(error.to_string().contains("--no-bump-context"));
}

#[test]
fn test_version_args_fixture() {
    let args = VersionArgsFixture::new().build();
    assert_eq!(args.main.source, sources::GIT);
    assert_eq!(args.main.output_format, formats::SEMVER);

    let args_with_overrides = VersionArgsFixture::new()
        .with_tag_version("v2.0.0")
        .with_distance(5)
        .with_dirty(true)
        .build();
    assert_eq!(
        args_with_overrides.overrides.tag_version,
        Some("v2.0.0".to_string())
    );
    assert_eq!(args_with_overrides.overrides.distance, Some(5));
    assert!(args_with_overrides.overrides.dirty);

    let args_with_clean = VersionArgsFixture::new().with_clean_flag(true).build();
    assert!(args_with_clean.overrides.clean);

    let args_with_bumps = VersionArgsFixture::new()
        .with_bump_major(1)
        .with_bump_minor(1)
        .with_bump_patch(1)
        .build();
    assert!(args_with_bumps.bumps.bump_major.is_some());
    assert!(args_with_bumps.bumps.bump_minor.is_some());
    assert!(args_with_bumps.bumps.bump_patch.is_some());
}

#[test]
fn test_validate_pre_release_flag_conflicts() {
    // Test conflicting pre-release flags
    let mut args = VersionArgsFixture::new()
        .with_pre_release_label("alpha")
        .with_bump_pre_release_label("beta")
        .build();
    let result = args.validate();
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(
        error,
        crate::error::ZervError::ConflictingOptions(_)
    ));
    assert!(error.to_string().contains("--pre-release-label"));
    assert!(error.to_string().contains("--bump-pre-release-label"));
    assert!(error.to_string().contains("Cannot use"));
}

#[test]
fn test_validate_pre_release_flags_no_conflict() {
    // Test that individual pre-release flags don't conflict
    let mut args = VersionArgsFixture::new()
        .with_pre_release_label("alpha")
        .build();
    assert_eq!(args.overrides.pre_release_label, Some("alpha".to_string()));
    assert_eq!(args.bumps.bump_pre_release_label, None);
    assert!(args.validate().is_ok());

    let mut args = VersionArgsFixture::new()
        .with_bump_pre_release_label("beta")
        .build();
    assert_eq!(args.overrides.pre_release_label, None);
    assert_eq!(args.bumps.bump_pre_release_label, Some("beta".to_string()));
    assert!(args.validate().is_ok());
}

#[test]
fn test_resolve_schema() {
    let args = VersionArgs::default();
    let (schema_name, schema_ron) = args.resolve_schema();
    assert_eq!(schema_name, Some("zerv-standard"));
    assert_eq!(schema_ron, None);
}
