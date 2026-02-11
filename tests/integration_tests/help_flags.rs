use rstest::rstest;
use zerv::config::EnvVars;

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
fn test_main_help_contains_examples() {
    let test_output = TestCommand::new().arg("--help").assert_success();
    let stdout = test_output.stdout();

    // Should contain comprehensive description
    assert!(
        stdout.contains("dynamic versioning tool"),
        "Should contain main description"
    );
    assert!(
        stdout.contains("version control system"),
        "Should mention VCS"
    );
    assert!(
        stdout.contains("configurable schemas"),
        "Should mention schemas"
    );

    // Should contain examples section
    assert!(
        stdout.contains("EXAMPLES:"),
        "Should contain examples section"
    );
    assert!(stdout.contains("zerv version"), "Should show basic usage");
    assert!(
        stdout.contains("--output-format pep440"),
        "Should show format example"
    );
    assert!(
        stdout.contains("--tag-version v2.0.0"),
        "Should show override example"
    );
    assert!(stdout.contains("--clean"), "Should show clean flag example");
    assert!(
        stdout.contains("Pipe") || stdout.contains("pipe"),
        "Should mention piping"
    );
    assert!(
        stdout.contains("-C /path/to/repo"),
        "Should show directory example"
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

    // Should contain detailed description
    assert!(
        stdout.contains("Generate version strings"),
        "Should contain detailed description"
    );
    assert!(
        stdout.contains("configurable schemas"),
        "Should mention schemas"
    );
    assert!(
        stdout.contains("multiple input sources"),
        "Should mention input sources"
    );
    assert!(stdout.contains("CI/CD workflows"), "Should mention CI/CD");

    // Should document input sources
    assert!(stdout.contains("git"), "Should document git source");
    assert!(stdout.contains("stdin"), "Should document stdin source");
    assert!(
        stdout.contains("Zerv RON format"),
        "Should mention RON format"
    );

    // Should document output formats
    assert!(stdout.contains("semver"), "Should document semver format");
    assert!(stdout.contains("pep440"), "Should document pep440 format");
    assert!(stdout.contains("zerv"), "Should document zerv format");

    // Should show possible values
    assert!(
        stdout.contains("[possible values: git, stdin, none]"),
        "Should show source values"
    );
    assert!(
        stdout.contains("[possible values: auto, semver, pep440]"),
        "Should show input format values"
    );
    assert!(
        stdout.contains("[possible values: semver, pep440, zerv]"),
        "Should show output format values"
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
    assert!(
        stdout.contains("Validate"),
        "Should contain validation description"
    );
}

#[rstest]
#[case(None, "default")]
#[case(Some(""), "empty pager")]
#[case(Some("nonexistent-pager"), "invalid pager")]
fn test_llm_help_flag(#[case] pager_env: Option<&str>, #[case] description: &str) {
    let test_output = match pager_env {
        Some(pager) => TestCommand::new()
            .env(EnvVars::PAGER, pager)
            .arg("--llm-help")
            .assert_success(),
        None => TestCommand::new().arg("--llm-help").assert_success(),
    };

    let stdout = test_output.stdout();

    assert!(
        stdout.contains("# zerv"),
        "Should contain manual title with {}: {stdout}",
        description
    );

    // Should have substantial content for the main test case
    if pager_env.is_none() {
        assert!(
            stdout.len() > 1000,
            "Manual should have substantial content with {} (got {} chars)",
            description,
            stdout.len()
        );
    } else {
        // For pager edge cases, just verify there's output
        assert!(
            !stdout.is_empty(),
            "Should have output content with {}",
            description
        );
    }
}

#[test]
fn test_invalid_command_shows_help() {
    let test_output = TestCommand::new().arg("invalid-command").assert_failure();
    let stderr = test_output.stderr();

    assert!(stderr.contains("error:"), "Should show error");
    assert!(
        stderr.contains("For more information, try '--help'"),
        "Should suggest help"
    );
}
