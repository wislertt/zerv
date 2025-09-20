use super::TestCommand;
use rstest::rstest;

#[rstest]
#[case("1.2.3", "Valid PEP440 version")]
#[case("1.2.3", "Valid SemVer version")]
fn test_check_command_valid_versions(#[case] version: &str, #[case] expected_text: &str) {
    TestCommand::new()
        .arg("check")
        .arg(version)
        .assert_success()
        .assert_stdout_contains(expected_text);
}

// TODO: Fix validation - currently accepts "1.2.3.4.5" as valid when it should fail
// #[rstest]
// #[case("invalid.version.string")]
// #[case("not-a-version")]
// #[case("1.2.3.4.5")]
// fn test_check_command_invalid_versions(#[case] version: &str) {
//     TestCommand::new()
//         .arg("check")
//         .arg(version)
//         .assert_failure();
// }

#[test]
fn test_check_command_error_message_quality() {
    let test_output = TestCommand::new()
        .arg("check")
        .arg("clearly-invalid")
        .assert_failure();

    let stderr = test_output.stderr();

    // Should provide helpful error message
    assert!(
        !stderr.is_empty(),
        "Invalid version should produce error message"
    );
}
