use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

/// Source tests for "zerv flow" command
/// Smart source default is covered by existing flow scenario tests

#[test]
fn test_flow_source_stdin() {
    // Test stdin source with flow
    let zerv_ron = ZervFixture::new()
        .with_version(2, 3, 4)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard --output-format semver",
        zerv_ron,
    );

    // Should work with stdin source
    assert_eq!(output.trim(), "2.3.4");
}

#[test]
fn test_flow_smart_default_with_stdin() {
    // Smart default: with stdin → defaults to stdin
    let zerv_ron = ZervFixture::new()
        .with_version(3, 4, 5)
        .with_branch("feature".to_string())
        .build()
        .to_string();

    let output =
        TestCommand::run_with_stdin("flow --schema standard --output-format semver", zerv_ron);

    // Should default to stdin source
    assert!(output.contains("3.4"), "Should output version from stdin");
}

#[test]
fn test_flow_source_none() {
    // Test --source none with flow
    let output = TestCommand::new()
        .args_from_str("flow --source none --tag-version 7.8.9 --output-format semver")
        .assert_success();

    assert_eq!(output.stdout().trim(), "7.8.9");
}
