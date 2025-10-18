use rstest::rstest;

use super::TestCommand;

#[rstest]
#[case("1.2.3")]
#[case("1.0.0")]
#[case("2.1.4")]
fn test_check_pep440_format(#[case] version: &str) {
    TestCommand::new()
        .arg("check")
        .arg(version)
        .arg("--format")
        .arg("pep440")
        .assert_success()
        .assert_stdout_contains("Valid PEP440 format");
}

#[rstest]
#[case("1.2.3")]
#[case("1.0.0")]
#[case("2.1.4")]
fn test_check_semver_format(#[case] version: &str) {
    TestCommand::new()
        .arg("check")
        .arg(version)
        .arg("--format")
        .arg("semver")
        .assert_success()
        .assert_stdout_contains("Valid SemVer format");
}

#[test]
fn test_check_unknown_format() {
    TestCommand::new()
        .arg("check")
        .arg("1.2.3")
        .arg("--format")
        .arg("unknown-format")
        .assert_failure();
}
