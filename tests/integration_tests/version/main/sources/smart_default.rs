use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

/// Smart source default: no explicit --source → defaults based on context
/// - With stdin → defaults to stdin
/// - Without stdin (in git repo) → defaults to git

#[test]
fn test_smart_default_with_stdin_pipe() {
    // With stdin → should default to stdin
    let zerv_ron = ZervFixture::new().with_version(2, 3, 4).build().to_string();

    let output = TestCommand::run_with_stdin("version --output-format semver", zerv_ron);

    // Should use stdin source (smart default)
    assert_eq!(output.trim(), "2.3.4");
}

#[test]
fn test_smart_default_in_git_repo() {
    // In git repo (running tests), no stdin → should default to git
    let output = TestCommand::new()
        .args_from_str("version --output-format semver")
        .assert_success();

    // Should use git source (smart default)
    let stdout = output.stdout();
    assert!(stdout.contains("."), "Should output version from git");
}

#[test]
fn test_explicit_source_overrides_stdin() {
    // Explicit --source should override smart default (stdin presence)
    let zerv_ron = ZervFixture::new().with_version(3, 4, 5).build().to_string();

    let output =
        TestCommand::run_with_stdin("version --source stdin --output-format semver", zerv_ron);

    // Explicit source wins (stdin content ignored for git detection)
    assert!(
        output.contains("3.4"),
        "Should use explicit source, not smart default"
    );
}

#[test]
fn test_smart_default_none_source() {
    // Explicit --source none uses only overrides
    let output = TestCommand::new()
        .args_from_str("version --source none --tag-version 5.6.7 --output-format semver")
        .assert_success();

    assert_eq!(output.stdout().trim(), "5.6.7");
}
