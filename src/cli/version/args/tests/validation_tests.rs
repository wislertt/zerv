use super::super::*;
use crate::error::ZervError;

/// Helper function to validate that an invalid input is rejected for a specific section
fn assert_invalid_input_rejected(invalid_input: &str, section: &str, expected_arg_name: &str) {
    let mut args = match section {
        "core" => VersionArgs {
            bumps: BumpsConfig {
                bump_core: vec![invalid_input.to_string().into()],
                ..Default::default()
            },
            ..Default::default()
        },
        "extra_core" => VersionArgs {
            bumps: BumpsConfig {
                bump_extra_core: vec![invalid_input.to_string().into()],
                ..Default::default()
            },
            ..Default::default()
        },
        "build" => VersionArgs {
            bumps: BumpsConfig {
                bump_build: vec![invalid_input.to_string().into()],
                ..Default::default()
            },
            ..Default::default()
        },
        _ => panic!("Unknown section: {section}"),
    };

    let result = args.validate();
    assert!(
        result.is_err(),
        "Should reject '{invalid_input}' in {section} section"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, ZervError::InvalidArgument(_)),
        "Should return InvalidArgument for '{invalid_input}' in {section} section"
    );

    assert!(
        error.to_string().contains(expected_arg_name),
        "Error message should contain '{expected_arg_name}' for '{invalid_input}' in {section} section. Got: {error}"
    );
    assert!(
        error
            .to_string()
            .contains("must be in format 'index[=value]'"),
        "Error message should mention format requirement for '{invalid_input}' in {section} section. Got: {error}"
    );
}

#[test]
fn test_validate_schema_bump_args_invalid_cases_cross_product() {
    let invalid_inputs = vec![
        "abc", "1.5", "1a", "a1", "1-", "-", "--1", "1--", "1 2", "1\t2", "1\n2", "1=2=3", "=1",
        "1=", "", " ", "\t", "\n", "1@2", "1#2", "1$2", "1%2", "1^2", "1&2", "1*2", "1+2", "1(2",
        "1)2", "1[2", "1]2", "1{2", "1}2", "1|2", "1\\2", "1:2", "1;2", "1'2", "1\"2", "1,2",
        "1<2", "1>2", "1?2", "1/2", "1~2", "1`2",
    ];
    let sections = vec![
        ("core", "--bump-core"),
        ("extra_core", "--bump-extra-core"),
        ("build", "--bump-build"),
    ];

    for invalid_input in invalid_inputs {
        for (section, expected_arg_name) in &sections {
            assert_invalid_input_rejected(invalid_input, section, expected_arg_name);
        }
    }
}

#[test]
fn test_validate_schema_bump_args_invalid_format() {
    // Test invalid schema bump argument formats
    let mut args = VersionArgs {
        bumps: BumpsConfig {
            bump_core: vec!["invalid_format".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = args.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, crate::error::ZervError::InvalidArgument(_)));
    assert!(
        error
            .to_string()
            .contains("must be in format 'index[=value]'")
    );
}

#[test]
fn test_validate_schema_bump_args_valid_formats() {
    // Test valid schema bump argument formats
    let mut args = VersionArgs {
        bumps: BumpsConfig {
            bump_core: vec!["0".into(), "1=5".into(), "-1=3".into()],
            bump_extra_core: vec!["0=release".into()],
            bump_build: vec!["-1".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = args.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_schema_bump_args_empty() {
    // Test empty schema bump arguments (should be valid)
    let mut args = VersionArgs {
        bumps: BumpsConfig {
            bump_core: vec![],
            bump_extra_core: vec![],
            bump_build: vec![],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = args.validate();
    assert!(result.is_ok());
    assert!(args.bumps.bump_core.is_empty());
    assert!(args.bumps.bump_extra_core.is_empty());
    assert!(args.bumps.bump_build.is_empty());
}

#[test]
fn test_validate_schema_bump_args_valid() {
    // Test valid schema bump arguments
    let mut args = VersionArgs {
        bumps: BumpsConfig {
            bump_core: vec!["0=1".into(), "2=3".into()],
            bump_extra_core: vec!["1=5".into()],
            bump_build: vec!["0=release".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = args.validate();
    assert!(result.is_ok());
}

#[test]
fn test_validate_schema_bump_args_invalid_odd_count() {
    // Test that single values (without =) are now valid (default to value 1)
    let mut args = VersionArgs {
        bumps: BumpsConfig {
            bump_core: vec!["0=1".into(), "2".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = args.validate();
    assert!(result.is_ok());
}
