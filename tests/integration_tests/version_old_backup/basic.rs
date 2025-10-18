use super::TestCommand;

#[test]
fn test_version_command_generates_version() {
    let test_output = TestCommand::new().arg("version").assert_success();

    let output = test_output.stdout();

    // Should contain version-like pattern (numbers and dots)
    assert!(
        output.contains('.'),
        "Version should contain dots: {output}"
    );
    assert!(
        output.chars().any(|c| c.is_ascii_digit()),
        "Version should contain numbers: {output}"
    );
}
