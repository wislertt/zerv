use super::TestCommand;
use rstest::rstest;

#[rstest]
#[case("1.2.3")]
#[case("2.0.0")]
fn test_check_auto_detect_both_formats(#[case] version: &str) {
    let test_output = TestCommand::new()
        .arg("check")
        .arg(version)
        .assert_success();

    let stdout = test_output.stdout();

    // Should detect both PEP440 and SemVer for simple versions
    assert!(
        stdout.contains("Valid PEP440 version") && stdout.contains("Valid SemVer version"),
        "Auto-detect should identify both formats for {version}: {stdout}"
    );
}

#[test]
fn test_check_auto_detect_pep440_only() {
    // Test version that's PEP440 but not SemVer (if such cases exist)
    let test_output = TestCommand::new()
        .arg("check")
        .arg("1.2.3.dev1")
        .assert_success();

    let stdout = test_output.stdout();

    // Should detect PEP440 only
    assert!(
        stdout.contains("Valid PEP440 version"),
        "Should detect PEP440 format: {stdout}"
    );
}

#[test]
fn test_check_auto_detect_semver_only() {
    // Test version that's SemVer but not PEP440 (if such cases exist)
    let test_output = TestCommand::new()
        .arg("check")
        .arg("1.2.3-alpha.1")
        .assert_success();

    let stdout = test_output.stdout();

    // Should detect SemVer only
    assert!(
        stdout.contains("Valid SemVer version"),
        "Should detect SemVer format: {stdout}"
    );
}
