// Error handling tests for flow command using stdin input

use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[test]
fn test_flow_command_empty_stdin() {
    let output = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --schema standard --output-format semver",
        "".to_string(),
    );

    // Should contain error about empty stdin content
    assert!(
        output.contains("empty") || output.contains("stdin") || output.contains("No stdin content"),
        "Error message should indicate empty input issue: {}",
        output
    );
}

#[test]
fn test_flow_command_invalid_stdin_format() {
    let output = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --schema standard --output-format semver",
        "invalid ron content".to_string(),
    );

    // Should contain error about invalid RON format
    assert!(
        output.contains("invalid") || output.contains("RON") || output.contains("parse"),
        "Error message should indicate RON parsing issue: {}",
        output
    );
}

#[test]
fn test_flow_command_invalid_source() {
    let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

    let output = TestCommand::run_with_stdin_expect_fail(
        "flow --source invalid_source --schema standard --output-format semver",
        zerv_ron,
    );

    // Should contain error about invalid source
    assert!(
        output.contains("invalid") || output.contains("source"),
        "Error message should indicate invalid source: {}",
        output
    );
}

#[test]
fn test_flow_command_conflicting_format_options() {
    let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

    let output = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --schema standard --output-format semver --output-format pep440",
        zerv_ron,
    );

    // Should contain error about conflicting format options
    assert!(
        output.contains("conflict") || output.contains("format") || output.contains("argument"),
        "Error message should indicate conflicting format options: {}",
        output
    );
}
