use crate::util::TestCommand;

#[test]
fn test_troubleshooting_unknown_schema_message() {
    // --source none keeps VCS out of the picture; the schema name is the failure
    let stderr = TestCommand::run_expect_fail(
        "version --source none --tag-version 1.0.0 --schema standard-bse",
    );
    assert!(
        stderr.contains("Unknown schema: standard-bse"),
        "expected Unknown schema message, got: {stderr}"
    );
}

#[test]
fn test_troubleshooting_conflicting_options_message() {
    let stderr = TestCommand::run_expect_fail(
        r#"render "1.2.3" --output-template "v{{major}}" --output-prefix "v""#,
    );
    assert!(
        stderr.contains("Conflicting options"),
        "expected Conflicting options message, got: {stderr}"
    );
    assert!(
        stderr.contains("Cannot use --output-template with --output-prefix"),
        "expected the template/prefix conflict detail, got: {stderr}"
    );
}
