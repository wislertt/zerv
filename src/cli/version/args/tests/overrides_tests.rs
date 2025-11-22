use clap::Parser;

use super::super::*;
use crate::cli::utils::template::Template;

#[test]
fn test_overrides_config_defaults() {
    let config = OverridesConfig::try_parse_from(["version"]).unwrap();

    // VCS override options should be None/false by default
    assert!(config.common.tag_version.is_none());
    assert!(config.common.distance.is_none());
    assert!(!config.common.dirty);
    assert!(!config.common.no_dirty);
    assert!(!config.common.clean);
    assert!(config.common.bumped_branch.is_none());
    assert!(config.common.bumped_commit_hash.is_none());

    // Version component overrides should be None by default
    assert!(config.common.major.is_none());
    assert!(config.common.minor.is_none());
    assert!(config.common.patch.is_none());
    assert!(config.common.epoch.is_none());
    assert!(config.common.post.is_none());
    assert!(config.dev.is_none());
    assert!(config.pre_release_label.is_none());
    assert!(config.pre_release_num.is_none());
    assert!(config.custom.is_none());
}

#[test]
fn test_overrides_config_with_values() {
    let config = OverridesConfig::try_parse_from([
        "zerv",
        "--tag-version",
        "v2.0.0",
        "--distance",
        "5",
        "--dirty",
        "--bumped-branch",
        "feature/test",
        "--bumped-commit-hash",
        "abc123",
        "--major",
        "2",
        "--minor",
        "1",
        "--patch",
        "0",
        "--pre-release-label",
        "alpha",
        "--pre-release-num",
        "1",
    ])
    .unwrap();

    assert_eq!(config.common.tag_version, Some("v2.0.0".to_string()));
    assert_eq!(config.common.distance, Some(5));
    assert!(config.common.dirty);
    assert!(!config.common.no_dirty);
    assert!(!config.common.clean);
    assert_eq!(
        config.common.bumped_branch,
        Some("feature/test".to_string())
    );
    assert_eq!(config.common.bumped_commit_hash, Some("abc123".to_string()));
    assert_eq!(config.common.major, Some(2.into()));
    assert_eq!(config.common.minor, Some(1.into()));
    assert_eq!(config.common.patch, Some(0.into()));
    assert_eq!(
        config.pre_release_label,
        Some(Template::new("alpha".to_string()))
    );
    assert_eq!(config.pre_release_num, Some(1.into()));
}

#[test]
fn test_overrides_config_clean_flag() {
    let config = OverridesConfig::try_parse_from(["version", "--clean"]).unwrap();

    assert!(config.common.clean);
    assert!(config.common.distance.is_none());
    assert!(!config.common.dirty);
    assert!(!config.common.no_dirty);
}

#[test]
fn test_overrides_config_dirty_flags() {
    // Test --dirty flag
    let config = OverridesConfig::try_parse_from(["version", "--dirty"]).unwrap();
    assert!(config.common.dirty);
    assert!(!config.common.no_dirty);

    // Test --no-dirty flag
    let config = OverridesConfig::try_parse_from(["version", "--no-dirty"]).unwrap();
    assert!(!config.common.dirty);
    assert!(config.common.no_dirty);
}

#[test]
fn test_dirty_override_helper() {
    // Test --dirty flag
    let config = OverridesConfig::try_parse_from(["version", "--dirty"]).unwrap();
    assert_eq!(config.dirty_override(), Some(true));

    // Test --no-dirty flag
    let config = OverridesConfig::try_parse_from(["version", "--no-dirty"]).unwrap();
    assert_eq!(config.dirty_override(), Some(false));

    // Test neither flag (use VCS)
    let config = OverridesConfig::try_parse_from(["version"]).unwrap();
    assert_eq!(config.dirty_override(), None);
}

#[test]
fn test_validate_overrides_no_conflicts() {
    // Test with no conflicting options
    let config = OverridesConfig::try_parse_from(["version"]).unwrap();
    assert!(Validation::validate_overrides(&config).is_ok());

    // Test with individual options (no conflicts)
    let config = OverridesConfig::try_parse_from(["version", "--dirty"]).unwrap();
    assert!(Validation::validate_overrides(&config).is_ok());

    let config = OverridesConfig::try_parse_from(["version", "--no-dirty"]).unwrap();
    assert!(Validation::validate_overrides(&config).is_ok());

    let config = OverridesConfig::try_parse_from(["version", "--clean"]).unwrap();
    assert!(Validation::validate_overrides(&config).is_ok());

    let config = OverridesConfig::try_parse_from(["version", "--distance", "5"]).unwrap();
    assert!(Validation::validate_overrides(&config).is_ok());
}

#[test]
fn test_validate_overrides_clean_with_non_conflicting_options() {
    // Test --clean with options that should NOT conflict
    let config = OverridesConfig::try_parse_from([
        "zerv",
        "--clean",
        "--tag-version",
        "v2.0.0",
        "--bumped-branch",
        "main",
        "--bumped-commit-hash",
        "abc123",
    ])
    .unwrap();
    assert!(Validation::validate_overrides(&config).is_ok());
}
