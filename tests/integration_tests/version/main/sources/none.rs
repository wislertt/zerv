use crate::util::TestCommand;

/// Tests for --source none (no VCS, use only overrides)

#[test]
fn test_none_source_with_tag_version() {
    // --source none with --tag-version uses only the override
    let output = TestCommand::new()
        .args_from_str("version --source none --tag-version 2.5.0 --output-format semver")
        .assert_success();

    assert_eq!(output.stdout().trim(), "2.5.0");
}

#[test]
fn test_none_source_with_distance() {
    // --source none with --distance adds distance
    let output = TestCommand::new()
        .args_from_str(
            "version --source none --tag-version 1.2.3 --distance 5 --output-format semver",
        )
        .assert_success();

    assert_eq!(output.stdout().trim(), "1.2.3+5");
}

#[test]
fn test_none_source_with_dirty() {
    // --source none with --dirty (without distance, dirty is shown in dev)
    let output = TestCommand::new()
        .args_from_str("version --source none --tag-version 3.4.5 --dirty --output-format pep440")
        .assert_success();

    assert_eq!(output.stdout().trim(), "3.4.5");
}

#[test]
fn test_none_source_with_dirty_and_distance() {
    // --source none with both --dirty and --distance (distance takes precedence)
    let output = TestCommand::new()
        .args_from_str(
            "version --source none --tag-version 3.4.5 --distance 2 --dirty --output-format pep440",
        )
        .assert_success();

    assert_eq!(output.stdout().trim(), "3.4.5+2");
}

#[test]
fn test_none_source_with_clean() {
    // --source none with --clean (no distance, not dirty)
    let output = TestCommand::new()
        .args_from_str("version --source none --tag-version 4.0.0 --clean --output-format semver")
        .assert_success();

    assert_eq!(output.stdout().trim(), "4.0.0");
}

#[test]
fn test_none_source_with_all_overrides() {
    // --source none with multiple overrides
    let output = TestCommand::new()
        .args_from_str(
            "version --source none --tag-version 5.6.7 --distance 10 --bumped-branch feature-x --bumped-commit-hash gabc123def --output-format semver",
        )
        .assert_success();

    assert_eq!(output.stdout().trim(), "5.6.7+feature.x.10.gabc123d");
}

#[test]
fn test_none_source_without_tag_version_defaults_to_zero() {
    // --source none without --tag-version defaults to 0.0.0
    let output = TestCommand::new()
        .args_from_str("version --source none --output-format semver")
        .assert_success();

    assert_eq!(output.stdout().trim(), "0.0.0");
}
