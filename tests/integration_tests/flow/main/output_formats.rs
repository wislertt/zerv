// Output format tests for flow command using stdin input

use rstest::rstest;
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[rstest]
#[case::semver("semver", "1.2.3")]
#[case::pep440("pep440", "1.2.3")]
fn test_flow_command_output_formats(#[case] output_format: &str, #[case] expected_version: &str) {
    let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {output_format}"),
        zerv_ron,
    );

    assert_eq!(output, expected_version);
}

#[test]
fn test_flow_command_invalid_output_format() {
    let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

    let output = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --schema standard --output-format invalid_format",
        zerv_ron,
    );

    // Check that output contains error message about invalid format
    assert!(
        output.contains("invalid_format") || output.contains("unknown"),
        "Error message should indicate invalid format: {}",
        output
    );
}
