use rstest::rstest;

use crate::util::TestCommand;

#[rstest]
#[case("-V")]
#[case("--version")]
fn test_version_flags(#[case] flag: &str) {
    TestCommand::new().arg(flag).assert_success();
}

#[rstest]
#[case("-h")]
#[case("--help")]
fn test_help_flags(#[case] flag: &str) {
    let test_output = TestCommand::new().arg(flag).assert_success();
    let stdout = test_output.stdout();

    // Should show available commands
    assert!(
        stdout.contains("version") && stdout.contains("check"),
        "Help should show available commands: {stdout}"
    );
}

#[test]
fn test_version_command_help() {
    let test_output = TestCommand::new()
        .arg("version")
        .arg("--help")
        .assert_success();

    let stdout = test_output.stdout();

    // Should show version command options
    assert!(
        stdout.contains("--output-format") || stdout.contains("--source"),
        "Version help should show command options: {stdout}"
    );
}

#[test]
fn test_check_command_help() {
    let test_output = TestCommand::new()
        .arg("check")
        .arg("--help")
        .assert_success();

    let stdout = test_output.stdout();

    // Should show check command options
    assert!(
        stdout.contains("--format") || stdout.contains("version"),
        "Check help should show command options: {stdout}"
    );
}
