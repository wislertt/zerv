use clap::Parser;

use super::super::*;

#[test]
fn test_bumps_config_defaults() {
    let config = BumpsConfig::try_parse_from(["version"]).unwrap();

    // Bump options should be None by default
    assert!(config.bump_major.is_none());
    assert!(config.bump_minor.is_none());
    assert!(config.bump_patch.is_none());
    assert!(config.bump_post.is_none());
    assert!(config.bump_dev.is_none());
    assert!(config.bump_pre_release_num.is_none());
    assert!(config.bump_epoch.is_none());
    assert!(config.bump_pre_release_label.is_none());

    // Schema-based bump options should be empty by default
    assert!(config.bump_core.is_empty());
    assert!(config.bump_extra_core.is_empty());
    assert!(config.bump_build.is_empty());

    // Context control options should be false by default
    assert!(!config.bump_context);
    assert!(!config.no_bump_context);
}

#[test]
fn test_bumps_config_with_values() {
    let config = BumpsConfig::try_parse_from([
        "zerv",
        "--bump-major",
        "1",
        "--bump-minor",
        "2",
        "--bump-patch",
        "3",
        "--bump-pre-release-label",
        "alpha",
        "--bump-context",
    ])
    .unwrap();

    assert_eq!(config.bump_major, Some(Some(1)));
    assert_eq!(config.bump_minor, Some(Some(2)));
    assert_eq!(config.bump_patch, Some(Some(3)));
    assert_eq!(config.bump_pre_release_label, Some("alpha".to_string()));
    assert!(config.bump_context);
    assert!(!config.no_bump_context);
}

#[test]
fn test_bumps_config_schema_based() {
    let config = BumpsConfig::try_parse_from([
        "version",
        "--bump-core",
        "0=1",
        "--bump-core",
        "2=3",
        "--bump-extra-core",
        "1=5",
        "--bump-build",
        "0=10",
        "--bump-build",
        "1=20",
    ])
    .unwrap();

    assert_eq!(config.bump_core, vec!["0=1", "2=3"]);
    assert_eq!(config.bump_extra_core, vec!["1=5"]);
    assert_eq!(config.bump_build, vec!["0=10", "1=20"]);
}

#[test]
fn test_validate_bumps_no_conflicts() {
    // Test with no conflicting options
    let config = BumpsConfig::try_parse_from(["version"]).unwrap();
    assert!(Validation::validate_bumps(&config).is_ok());

    // Test with individual options (no conflicts)
    let config = BumpsConfig::try_parse_from(["version", "--bump-context"]).unwrap();
    assert!(Validation::validate_bumps(&config).is_ok());

    let config = BumpsConfig::try_parse_from(["version", "--no-bump-context"]).unwrap();
    assert!(Validation::validate_bumps(&config).is_ok());
}

#[test]
fn test_validate_bumps_context_control_conflicts() {
    // Test conflicting context control flags
    let config =
        BumpsConfig::try_parse_from(["version", "--bump-context", "--no-bump-context"]).unwrap();
    let result = Validation::validate_bumps(&config);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(
        error,
        crate::error::ZervError::ConflictingOptions(_)
    ));
    assert!(error.to_string().contains("--bump-context"));
    assert!(error.to_string().contains("--no-bump-context"));
    assert!(error.to_string().contains("conflicting options"));
}

#[test]
fn test_validate_bumps_schema_bump_args_valid() {
    // Test valid schema bump arguments (pairs of index, value)
    let config = BumpsConfig::try_parse_from([
        "version",
        "--bump-core",
        "0=1",
        "--bump-core",
        "2=3",
        "--bump-extra-core",
        "1=5",
        "--bump-build",
        "0=10",
        "--bump-build",
        "1=20",
    ])
    .unwrap();

    assert!(Validation::validate_bumps(&config).is_ok());
    assert_eq!(config.bump_core, vec!["0=1", "2=3"]);
    assert_eq!(config.bump_extra_core, vec!["1=5"]);
    assert_eq!(config.bump_build, vec!["0=10", "1=20"]);
}

#[test]
fn test_validate_bumps_schema_bump_args_invalid_odd_count() {
    // Test invalid schema bump arguments (odd number of arguments)
    // We need to manually create the config with odd count since clap validates pairs
    let config = BumpsConfig {
        bump_core: vec!["0=1".to_string(), "2".to_string()], // Odd count: 2 elements (one without value)
        ..Default::default()
    };
    let result = Validation::validate_bumps(&config);
    assert!(result.is_ok()); // Now supports single values without '='
}

#[test]
fn test_validate_bumps_schema_bump_args_empty() {
    // Test empty schema bump arguments (should be valid)
    let config = BumpsConfig::try_parse_from(["version"]).unwrap();
    assert!(Validation::validate_bumps(&config).is_ok());
    assert!(config.bump_core.is_empty());
    assert!(config.bump_extra_core.is_empty());
    assert!(config.bump_build.is_empty());
}

#[test]
fn test_resolve_context_control_defaults() {
    // Test all 4 possible states of (bump_context, no_bump_context)

    // Scenario 1: (false, false) - Neither flag provided: should default to bump-context
    let mut config = BumpsConfig::try_parse_from(["version"]).unwrap();
    assert!(!config.bump_context);
    assert!(!config.no_bump_context);
    assert!(Validation::resolve_context_control_defaults(&mut config).is_ok());
    assert!(config.bump_context);
    assert!(!config.no_bump_context);

    // Scenario 2: (true, false) - Explicit --bump-context: should remain unchanged
    let mut config = BumpsConfig::try_parse_from(["version", "--bump-context"]).unwrap();
    assert!(config.bump_context);
    assert!(!config.no_bump_context);
    assert!(Validation::resolve_context_control_defaults(&mut config).is_ok());
    assert!(config.bump_context);
    assert!(!config.no_bump_context);

    // Scenario 3: (false, true) - Explicit --no-bump-context: should remain unchanged
    let mut config = BumpsConfig::try_parse_from(["version", "--no-bump-context"]).unwrap();
    assert!(!config.bump_context);
    assert!(config.no_bump_context);
    assert!(Validation::resolve_context_control_defaults(&mut config).is_ok());
    assert!(!config.bump_context);
    assert!(config.no_bump_context);

    // Scenario 4: (true, true) - Both flags provided: should return error
    let mut config =
        BumpsConfig::try_parse_from(["version", "--bump-context", "--no-bump-context"]).unwrap();
    assert!(config.bump_context);
    assert!(config.no_bump_context);
    let result = Validation::resolve_context_control_defaults(&mut config);
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
fn test_resolve_bump_defaults() {
    let mut config = BumpsConfig {
        bump_major: Some(None),
        bump_minor: Some(None),
        bump_patch: Some(None),
        bump_post: Some(None),
        bump_dev: Some(None),
        bump_pre_release_num: Some(None),
        bump_epoch: Some(None),
        ..Default::default()
    };

    assert!(Validation::resolve_bump_defaults(&mut config).is_ok());

    // All Some(None) should become Some(Some(1))
    assert_eq!(config.bump_major, Some(Some(1)));
    assert_eq!(config.bump_minor, Some(Some(1)));
    assert_eq!(config.bump_patch, Some(Some(1)));
    assert_eq!(config.bump_post, Some(Some(1)));
    assert_eq!(config.bump_dev, Some(Some(1)));
    assert_eq!(config.bump_pre_release_num, Some(Some(1)));
    assert_eq!(config.bump_epoch, Some(Some(1)));
}
