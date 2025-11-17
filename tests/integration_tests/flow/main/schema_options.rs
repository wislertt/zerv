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
