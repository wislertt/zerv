use crate::integration_tests::util::command::TestCommand;
use rstest::rstest;

/// Test comprehensive CLI help text and error message consistency
/// This validates requirements 9.1, 9.2, 9.3, 9.4, 9.5, 9.6 from the CLI consistency requirements
/// Helper struct to mimic the expected result format
struct CommandResult {
    success: bool,
    stdout: String,
    stderr: String,
}

/// Helper function to run zerv command and return result
fn run_zerv_command(args: &[&str]) -> CommandResult {
    let mut cmd = TestCommand::new();
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().expect("Failed to execute command");
    CommandResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

#[test]
fn test_main_help_contains_examples() {
    let result = run_zerv_command(&["--help"]);
    assert!(result.success, "Help command should succeed");

    let output = result.stdout;

    // Should contain comprehensive description
    assert!(
        output.contains("dynamic versioning tool"),
        "Should contain main description"
    );
    assert!(
        output.contains("version control system"),
        "Should mention VCS"
    );
    assert!(
        output.contains("configurable schemas"),
        "Should mention schemas"
    );

    // Should contain examples section
    assert!(
        output.contains("EXAMPLES:"),
        "Should contain examples section"
    );
    assert!(output.contains("zerv version"), "Should show basic usage");
    assert!(
        output.contains("--output-format pep440"),
        "Should show format example"
    );
    assert!(
        output.contains("--tag-version v2.0.0"),
        "Should show override example"
    );
    assert!(output.contains("--clean"), "Should show clean flag example");
    assert!(
        output.contains("Pipe") || output.contains("pipe"),
        "Should mention piping"
    );
    assert!(
        output.contains("-C /path/to/repo"),
        "Should show directory example"
    );
}

#[test]
fn test_version_help_comprehensive() {
    let result = run_zerv_command(&["version", "--help"]);
    assert!(result.success, "Version help should succeed");

    let output = result.stdout;

    // Should contain detailed description
    assert!(
        output.contains("Generate version strings"),
        "Should contain detailed description"
    );
    assert!(
        output.contains("configurable schemas"),
        "Should mention schemas"
    );
    assert!(
        output.contains("multiple input sources"),
        "Should mention input sources"
    );
    assert!(output.contains("CI/CD workflows"), "Should mention CI/CD");

    // Should document input sources
    assert!(output.contains("git"), "Should document git source");
    assert!(output.contains("stdin"), "Should document stdin source");
    assert!(
        output.contains("Zerv RON format"),
        "Should mention RON format"
    );

    // Should document output formats
    assert!(output.contains("semver"), "Should document semver format");
    assert!(output.contains("pep440"), "Should document pep440 format");
    assert!(output.contains("zerv"), "Should document zerv format");

    // Should document VCS overrides
    assert!(
        output.contains("Override detected tag version"),
        "Should document tag override"
    );
    assert!(
        output.contains("Override distance from tag"),
        "Should document distance override"
    );
    assert!(
        output.contains("Override dirty state"),
        "Should document dirty override"
    );
    assert!(
        output.contains("Force clean release state"),
        "Should document clean flag"
    );
    assert!(
        output.contains("Override current branch name"),
        "Should document branch override"
    );
    assert!(
        output.contains("Override commit hash"),
        "Should document hash override"
    );

    // Should document boolean values
    assert!(
        output.contains("true/false, yes/no, y/n, 1/0, on/off"),
        "Should document boolean values"
    );
    assert!(
        output.contains("case-insensitive"),
        "Should mention case insensitivity"
    );

    // Should document conflicts
    assert!(
        output.contains("Conflicts with --distance and --dirty"),
        "Should document conflicts"
    );

    // Should show possible values
    assert!(
        output.contains("[possible values: git, stdin]"),
        "Should show source values"
    );
    assert!(
        output.contains("[possible values: auto, semver, pep440, zerv]"),
        "Should show input format values"
    );
    assert!(
        output.contains("[possible values: semver, pep440, zerv]"),
        "Should show output format values"
    );
}

#[test]
fn test_check_help_available() {
    let result = run_zerv_command(&["check", "--help"]);
    assert!(result.success, "Check help should succeed");

    let output = result.stdout;
    assert!(
        output.contains("Validate"),
        "Should contain validation description"
    );
}

#[test]
fn test_version_flag_shows_version() {
    let result = run_zerv_command(&["--version"]);
    assert!(result.success, "Version flag should succeed");

    let output = result.stdout;
    assert!(output.contains("zerv"), "Should contain program name");
    // Version should be from Cargo.toml
    assert!(!output.trim().is_empty(), "Should not be empty");
}

#[test]
fn test_invalid_command_shows_help() {
    let result = run_zerv_command(&["invalid-command"]);
    assert!(!result.success, "Invalid command should fail");

    let stderr = result.stderr;
    assert!(stderr.contains("error:"), "Should show error");
    assert!(
        stderr.contains("For more information, try '--help'"),
        "Should suggest help"
    );
}

#[rstest]
#[case("--output-format", "unknown", "possible values: semver, pep440, zerv")]
#[case("--source", "unknown", "possible values: git, stdin")]
#[case("--input-format", "unknown", "possible values: auto, semver, pep440")]
fn test_unknown_option_value_errors(
    #[case] option: &str,
    #[case] value: &str,
    #[case] expected_possible_values: &str,
) {
    let result = run_zerv_command(&["version", option, value]);
    assert!(!result.success, "Unknown {option} should fail");

    let stderr = result.stderr;
    assert!(
        stderr.contains(&format!("invalid value '{value}'")),
        "Should show invalid value"
    );
    assert!(
        stderr.contains(expected_possible_values),
        "Should show possible values"
    );
    assert!(
        stderr.contains("For more information, try '--help'"),
        "Should suggest help"
    );
}

#[test]
fn test_invalid_boolean_value_error() {
    let result = run_zerv_command(&["version", "--dirty", "maybe"]);
    assert!(!result.success, "Invalid boolean should fail");

    let stderr = result.stderr;
    assert!(
        stderr.contains("invalid value 'maybe'"),
        "Should show invalid value"
    );
    assert!(
        stderr.contains("Invalid boolean value"),
        "Should show boolean error"
    );
    assert!(
        stderr.contains("Supported values: true/false, t/f, yes/no, y/n, 1/0, on/off"),
        "Should show supported values"
    );
    assert!(
        stderr.contains("case-insensitive"),
        "Should mention case insensitivity"
    );
    assert!(
        stderr.contains("For more information, try '--help'"),
        "Should suggest help"
    );
}

#[rstest]
#[case("true")]
#[case("t")]
#[case("yes")]
#[case("y")]
#[case("1")]
#[case("on")]
#[case("TRUE")]
#[case("Yes")]
#[case("ON")]
#[case("false")]
#[case("f")]
#[case("no")]
#[case("n")]
#[case("0")]
#[case("off")]
#[case("FALSE")]
#[case("No")]
#[case("OFF")]
fn test_boolean_values_accepted(#[case] value: &str) {
    let result = run_zerv_command(&["version", "--dirty", value, "--tag-version", "1.0.0"]);
    // Command may fail for other reasons (no git repo), but should not fail on boolean parsing
    if !result.success {
        assert!(
            !result.stderr.contains("Invalid boolean value"),
            "Should accept boolean value '{}', but got error: {}",
            value,
            result.stderr
        );
    }
}

#[rstest]
#[case(&["--clean", "--distance", "5"], "--distance")]
#[case(&["--clean", "--dirty", "true"], "--dirty")]
fn test_conflicting_options_error(#[case] args: &[&str], #[case] conflicting_flag: &str) {
    let mut command_args = vec!["version"];
    command_args.extend(args);
    let result = run_zerv_command(&command_args);
    assert!(!result.success, "Conflicting options should fail");

    let stderr = result.stderr;
    assert!(
        stderr.contains("Conflicting options"),
        "Should show conflicting options error"
    );
    assert!(stderr.contains("--clean"), "Should mention clean flag");
    assert!(
        stderr.contains(conflicting_flag),
        "Should mention {conflicting_flag} flag"
    );
}

#[test]
fn test_help_shows_deprecated_version_arg() {
    let result = run_zerv_command(&["version", "--help"]);
    assert!(result.success, "Help should succeed");

    let output = result.stdout;
    assert!(
        output.contains("deprecated"),
        "Should mark version arg as deprecated"
    );
    assert!(
        output.contains("use --tag-version instead"),
        "Should suggest alternative"
    );
}

#[test]
fn test_help_shows_future_extension_options() {
    let result = run_zerv_command(&["version", "--help"]);
    assert!(result.success, "Help should succeed");

    let output = result.stdout;
    assert!(
        output.contains("future extension"),
        "Should mark future extension options"
    );
    assert!(
        output.contains("--output-template"),
        "Should show template option"
    );
}

#[test]
fn test_help_shows_examples_for_overrides() {
    let result = run_zerv_command(&["version", "--help"]);
    assert!(result.success, "Help should succeed");

    let output = result.stdout;

    // Should show examples for tag version
    assert!(
        output.contains("'v2.0.0'"),
        "Should show tag version example"
    );
    assert!(
        output.contains("'1.5.0-beta.1'"),
        "Should show prerelease example"
    );

    // Should show examples for prefix
    assert!(
        output.contains("'v' for 'v1.0.0'"),
        "Should show prefix example"
    );
}

#[rstest]
#[case("--output-format", "xyz", "possible values:")]
#[case("--source", "xyz", "possible values:")]
#[case("--dirty", "xyz", "Supported values:")]
fn test_error_message_consistency(
    #[case] option: &str,
    #[case] value: &str,
    #[case] expected_values_text: &str,
) {
    let result = run_zerv_command(&["version", option, value]);
    assert!(!result.success, "Should fail");

    let stderr = result.stderr;
    assert!(
        stderr.contains(&format!("invalid value '{value}'")),
        "Should show specific invalid value"
    );
    assert!(
        stderr.contains(expected_values_text),
        "Should show {expected_values_text} values"
    );
}

#[test]
fn test_backward_compatibility_patterns() {
    // Test that existing command patterns still work

    // Basic version command should work (may fail due to no git repo, but not due to CLI parsing)
    let result = run_zerv_command(&["version"]);
    if !result.success {
        // Should not fail due to CLI parsing issues
        assert!(
            !result.stderr.contains("error: "),
            "Should not have CLI parsing errors"
        );
    }

    // Schema option should work
    let result = run_zerv_command(&["version", "--schema", "standard"]);
    if !result.success {
        // Should not fail due to CLI parsing issues
        assert!(
            !result.stderr.contains("error: "),
            "Should not have CLI parsing errors"
        );
    }

    // Output format should work
    let result = run_zerv_command(&["version", "--output-format", "pep440"]);
    if !result.success {
        // Should not fail due to CLI parsing issues
        assert!(
            !result.stderr.contains("error: "),
            "Should not have CLI parsing errors"
        );
    }
}

#[test]
fn test_short_help_vs_long_help() {
    // Test short help (-h)
    let short_result = run_zerv_command(&["version", "-h"]);
    assert!(short_result.success, "Short help should succeed");

    // Test long help (--help)
    let long_result = run_zerv_command(&["version", "--help"]);
    assert!(long_result.success, "Long help should succeed");

    // Long help should contain more information
    assert!(
        long_result.stdout.len() >= short_result.stdout.len(),
        "Long help should be at least as detailed as short help"
    );

    // Both should contain basic information
    assert!(
        short_result.stdout.contains("Generate version"),
        "Short help should contain description"
    );
    assert!(
        long_result.stdout.contains("Generate version"),
        "Long help should contain description"
    );
}

#[test]
fn test_help_mentions_all_supported_formats() {
    let result = run_zerv_command(&["version", "--help"]);
    assert!(result.success, "Help should succeed");

    let output = result.stdout;

    // Should mention all supported formats
    assert!(output.contains("semver"), "Should mention semver");
    assert!(output.contains("pep440"), "Should mention pep440");
    assert!(output.contains("zerv"), "Should mention zerv");

    // Should explain what each format is for
    assert!(output.contains("semver"), "Should mention semver");
    assert!(output.contains("pep440"), "Should mention pep440");
    assert!(
        output.contains("RON format for piping"),
        "Should explain zerv format"
    );
}

#[test]
fn test_help_explains_piping_workflow() {
    let result = run_zerv_command(&["--help"]);
    assert!(result.success, "Help should succeed");

    let output = result.stdout;

    // Should contain piping examples
    assert!(
        output.contains("Pipe") || output.contains("pipe"),
        "Should mention piping"
    );
    assert!(
        output.contains("--output-format zerv"),
        "Should show zerv output for piping"
    );
    assert!(
        output.contains("--source stdin"),
        "Should show stdin input for piping"
    );
    assert!(output.contains("|"), "Should show pipe operator");
}
