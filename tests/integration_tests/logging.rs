//! Integration tests for logging functionality
//!
//! These tests verify that logging works correctly and doesn't interfere with normal operations.

use crate::integration_tests::util::command::TestCommand;

#[test]
fn test_verbose_flag_doesnt_crash() {
    let output = TestCommand::new()
        .args(["version", "--verbose"])
        .output()
        .expect("Failed to run zerv");

    assert!(output.status.success(), "Should succeed with --verbose");

    // Should produce version output on stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Should have version output on stdout"
    );

    // Should have debug logs on stderr (check for common log patterns)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("DEBUG") || stderr.contains("Running git command"),
        "Should have debug logs on stderr"
    );
}

#[test]
fn test_verbose_flag_short_form() {
    let output = TestCommand::new()
        .args(["version", "-v"])
        .output()
        .expect("Failed to run zerv");

    assert!(output.status.success(), "Should succeed with -v");

    // Should produce version output on stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Should have version output on stdout"
    );
}

#[test]
fn test_default_behavior_no_logs() {
    let output = TestCommand::new()
        .args(["version"])
        .output()
        .expect("Failed to run zerv");

    assert!(output.status.success(), "Should succeed without verbose");

    // Should produce version output on stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Should have version output on stdout"
    );

    // Should NOT have debug logs on stderr (only build/compile messages)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("DEBUG") && !stderr.contains("Running git command"),
        "Should not have debug logs on stderr by default"
    );
}

#[test]
fn test_verbose_with_stdin_piping() {
    // Generate zerv format output with verbose logging
    let zerv_output = TestCommand::new()
        .args(["version", "--verbose", "--output-format", "zerv"])
        .assert_success()
        .stdout();

    // Use the zerv output as stdin input to another command with verbose logging
    let result = TestCommand::new()
        .args([
            "version",
            "--verbose",
            "--source",
            "stdin",
            "--output-format",
            "semver",
        ])
        .stdin(zerv_output)
        .assert_success();

    // Should produce semver output
    let stdout = result.stdout();
    assert!(
        !stdout.trim().is_empty(),
        "Should have version output from second command"
    );

    // Should have debug logs in stderr
    let stderr = result.stderr();
    assert!(
        stderr.contains("DEBUG") || stderr.contains("Running git command"),
        "Should have debug logs from second command"
    );
}

#[test]
fn test_run_with_stdin_convenience_method() {
    // Generate zerv format output first
    let zerv_output = TestCommand::new()
        .args(["version", "--output-format", "zerv"])
        .assert_success()
        .stdout();

    // Use the convenience method to run with stdin input
    let result =
        TestCommand::run_with_stdin("version --source stdin --output-format semver", zerv_output);

    // Should produce semver output
    assert!(
        !result.trim().is_empty(),
        "Should have version output using run_with_stdin"
    );
}
