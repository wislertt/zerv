// Schema option tests for flow command using stdin input

use rstest::rstest;
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[rstest]
#[case::standard("standard", "1.2.3")]
#[case::standard_base("standard-base", "1.2.3")]
fn test_flow_command_schema_options(#[case] schema: &str, #[case] expected_version: &str) {
    let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema {schema} --output-format semver"),
        zerv_ron,
    );

    assert_eq!(output, expected_version);
}

#[test]
fn test_flow_command_invalid_schema() {
    let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

    let output = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --schema invalid_schema --output-format semver",
        zerv_ron,
    );

    // Check that output contains error message about invalid schema
    assert!(
        output.contains("invalid") || output.contains("unknown") || output.contains("schema"),
        "Error message should indicate invalid schema: {}",
        output
    );
}

#[test]
fn test_flow_command_missing_schema() {
    let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

    let output =
        TestCommand::run_with_stdin("flow --source stdin --output-format semver", zerv_ron);

    // Should work with default schema
    assert_eq!(output, "1.0.0");
}

#[test]
fn test_flow_command_schema_ron() {
    let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

    // Test with a custom schema that includes major.minor.patch
    let schema_ron = r#"(
        core: [var(Major), var(Minor), var(Patch)],
        extra_core: [],
        build: [],
    )"#;

    let output = TestCommand::run_with_stdin(
        &format!(
            "flow --source stdin --schema-ron '{}' --output-format semver",
            schema_ron
        ),
        zerv_ron,
    );

    assert_eq!(output, "1.2.3");
}

#[test]
fn test_flow_command_schema_ron_with_extra_core() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(zerv::version::zerv::PreReleaseLabel::Alpha, Some(5))
        .with_post(7)
        .build()
        .to_string();

    // Test with a custom schema that includes core and extra_core components
    let schema_ron = r#"(
        core: [var(Major), var(Minor), var(Patch)],
        extra_core: [var(PreRelease), var(Post)],
        build: [],
    )"#;

    let output = TestCommand::run_with_stdin(
        &format!(
            "flow --source stdin --schema-ron '{}' --output-format semver",
            schema_ron
        ),
        zerv_ron,
    );

    assert_eq!(output, "1.2.3-alpha.5.post.7");
}

#[test]
fn test_flow_command_schema_ron_invalid() {
    let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

    // Test with invalid RON schema
    let invalid_schema_ron = "invalid ron";

    let output = TestCommand::run_with_stdin_expect_fail(
        &format!(
            "flow --source stdin --schema-ron '{}' --output-format semver",
            invalid_schema_ron
        ),
        zerv_ron,
    );

    // Check that output contains error message about invalid RON schema
    assert!(
        output.contains("Invalid RON schema") || output.contains("Failed to parse"),
        "Error message should indicate invalid RON schema: {}",
        output
    );
}
