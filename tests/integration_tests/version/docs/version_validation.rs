// Documentation tests for zerv check command
// Tests ensure that README.md examples work as documented

use crate::util::TestCommand;

#[test]
fn test_zerv_check_documentation_examples() {
    // Test case 1: Check valid SemVer format
    let output = TestCommand::new()
        .args_from_str("check --format semver 1.0.0-rc.1.something.complex+something.complex")
        .assert_success()
        .stdout();
    assert_eq!(
        output,
        "Version: 1.0.0-rc.1.something.complex+something.complex\n✓ Valid SemVer format\n"
    );

    // Test case 2: Check valid PEP440 format
    let output = TestCommand::new()
        .args_from_str("check --format pep440 1.0.0a2.post5.dev3+something.complex")
        .assert_success()
        .stdout();
    assert_eq!(
        output,
        "Version: 1.0.0a2.post5.dev3+something.complex\n✓ Valid PEP440 format\n"
    );

    // Test case 3: Check valid PEP440 format
    let output = TestCommand::new()
        .args_from_str("check --format pep440 1.0.0-alpha.2.post.5.dev.3+something.complex")
        .assert_success()
        .stdout();
    assert_eq!(
        output,
        "Version: 1.0.0-alpha.2.post.5.dev.3+something.complex\n✓ Valid PEP440 format (normalized: 1.0.0a2.post5.dev3+something.complex)\n"
    );

    // Test case 4: Check invalid version (should fail)
    let result = TestCommand::new()
        .args_from_str("check --format semver invalid")
        .assert_failure()
        .stderr();
    assert_eq!(
        result,
        "Error: Invalid version: invalid - Invalid SemVer format\n"
    );

    // Test case 5: Check auto-detected format
    let output = TestCommand::new()
        .args_from_str("check 2.1.0-beta.1")
        .assert_success()
        .stdout();
    assert_eq!(
        output,
        "Version: 2.1.0-beta.1\n✓ Valid PEP440 format (normalized: 2.1.0b1)\n✓ Valid SemVer format\n"
    );
}
