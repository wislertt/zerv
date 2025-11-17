// Basic flow command integration tests

// Tests for basic flow command functionality using stdin input
// Similar to version command integration tests

use rstest::rstest;
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

#[rstest]
#[case::basic_semver((1, 2, 3), "semver", "main", "1.2.3")]
#[case::basic_pep440((2, 0, 0), "pep440", "main", "2.0.0")]
fn test_basic_flow_command_with_stdin(
    #[case] version: (u64, u64, u64),
    #[case] format: &str,
    #[case] branch: &str,
    #[case] expected: &str,
) {
    let zerv_ron = ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_branch(branch.to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[rstest]
#[case::alpha_semver(PreReleaseLabel::Alpha, Some(1), "semver", "1.0.0-alpha.1")]
#[case::beta_pep440(PreReleaseLabel::Beta, Some(2), "pep440", "1.0.0b2")]
fn test_flow_command_with_pre_release_stdin(
    #[case] label: PreReleaseLabel,
    #[case] number: Option<u64>,
    #[case] format: &str,
    #[case] expected: &str,
) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 0, 0)
        .with_pre_release(label, number)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[rstest]
#[case::feature_branch("feature/test", "semver", "1.0.0")]
#[case::develop_branch("develop", "pep440", "1.0.0")]
fn test_flow_command_with_branch_name_stdin(
    #[case] branch: &str,
    #[case] format: &str,
    #[case] expected: &str,
) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 0, 0)
        .with_branch(branch.to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[rstest]
#[case::different_versions((1, 0, 0), (1, 0, 1), "semver", "1.0.0")]
#[case::pep440_bump((2, 1, 0), (2, 2, 0), "pep440", "2.1.0")]
fn test_flow_command_with_different_versions_stdin(
    #[case] current_version: (u64, u64, u64),
    #[case] _bumped_version: (u64, u64, u64),
    #[case] format: &str,
    #[case] expected: &str,
) {
    let zerv_ron = ZervFixture::new()
        .with_version(current_version.0, current_version.1, current_version.2)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[test]
fn test_flow_command_help() {
    let output = TestCommand::run_with_stdin("flow --help", "".to_string());

    assert!(output.contains("flow"));
    assert!(output.contains("branch patterns"));
    assert!(output.contains("pre-release detection"));
}

#[test]
fn test_flow_command_requires_source() {
    // Test that flow command works (defaults to git source)
    let mut cmd = TestCommand::new();
    cmd.args_from_str("flow -C ."); // Use -C to explicitly set git directory to current workspace root

    let result = cmd.output().expect("Failed to execute flow command");

    // The flow command should work successfully (defaults to git source)
    // TODO: Remove debug output after CI fix is verified
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);
        let exit_code = result.status.code();
        panic!(
            "Flow command failed!\nExit code: {:?}\nSTDERR:\n{}\nSTDOUT:\n{}",
            exit_code, stderr, stdout
        );
    }
}

#[test]
fn test_flow_command_invalid_option() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 0, 0)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let result =
        TestCommand::run_with_stdin_expect_fail("flow --source stdin --invalid-option", zerv_ron);

    // Should fail due to invalid option
    assert!(result.contains("error") || result.contains("unexpected"));
}

#[rstest]
#[case::semver_with_pre_release_number("semver", "1.0.0-alpha.1")]
#[case::pep440_with_dev("pep440", "1.0.0a1")]
fn test_flow_command_with_pre_release_number_stdin(#[case] format: &str, #[case] expected: &str) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 0, 0)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[test]
fn test_flow_command_empty_stdin() {
    // Test that empty stdin results in appropriate error
    let result = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --output-format semver",
        "".to_string(),
    );

    assert!(result.contains("error") || result.contains("failed"));
}

#[test]
fn test_flow_command_invalid_stdin_format() {
    // Test that invalid RON format results in appropriate error
    let result = TestCommand::run_with_stdin_expect_fail(
        "flow --source stdin --output-format semver",
        "invalid ron content".to_string(),
    );

    assert!(result.contains("error") || result.contains("failed"));
}
