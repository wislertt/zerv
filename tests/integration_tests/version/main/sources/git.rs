use zerv::schema::ZervSchemaPreset;
use zerv::test_utils::{
    GitRepoFixture,
    TestDir,
    ZervFixture,
    should_run_docker_tests,
};
use zerv::version::Zerv;

use crate::util::TestCommand;

/// Comprehensive git integration test covering the full pipeline:
/// Git → VCS Detection → Version Parsing → RON Serialization → Deserialization → Validation
#[test]
fn test_git_source_comprehensive() {
    if !should_run_docker_tests() {
        return;
    }

    // Create git fixture with dirty state (tagged v1.2.3 + uncommitted changes)
    let fixture = GitRepoFixture::dirty("v1.2.3").expect("Failed to create git repository");

    // Execute zerv: git source → zerv RON output
    let output = TestCommand::new()
        .current_dir(fixture.path())
        .args_from_str("version --source git --output-format zerv")
        .assert_success();

    // Parse output back to Zerv object
    let parsed_zerv: Zerv =
        ron::from_str(output.stdout().trim()).expect("Failed to parse output as Zerv");

    // Fuzzy check: commit hash should exist and be valid hex
    assert!(
        parsed_zerv.vars.bumped_commit_hash.is_some(),
        "Commit hash should be present"
    );
    if let Some(ref hash) = parsed_zerv.vars.bumped_commit_hash {
        assert!(
            hash.len() >= 7 && hash.len() <= 40,
            "Commit hash should be 7-40 chars"
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Commit hash should be hex"
        );
    }

    // Build expected Zerv object with VCS data
    let expected = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostDevContext)
        .with_vcs_data(
            Some(0),
            Some(true),
            Some("main".to_string()),
            None, // non-deterministic variable
            None,
            None, // non-deterministic variables
            None,
        )
        .build();

    // Copy non-deterministic timestamp
    let mut expected = expected;
    expected.vars.bumped_commit_hash = parsed_zerv.vars.bumped_commit_hash.clone();
    expected.vars.last_timestamp = parsed_zerv.vars.last_timestamp;
    expected.vars.bumped_timestamp = parsed_zerv.vars.bumped_timestamp;

    // Git source doesn't provide last_branch - it should be None
    assert_eq!(
        parsed_zerv.vars.last_branch, None,
        "Git source should not provide last_branch"
    );

    // Assert the entire Zerv object matches expected
    assert_eq!(
        parsed_zerv, expected,
        "Parsed Zerv should match expected structure"
    );
}

#[test]
fn test_git_source_not_a_git_repo() {
    let test_dir = TestDir::new().expect("Failed to create test directory");

    let output = TestCommand::new()
        .current_dir(test_dir.path())
        .args_from_str("version --source git")
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains("Error: VCS not found: Not in a git repository (--source git)"),
        "stderr should contain expected error message. Got: {stderr}"
    );
}

#[test]
fn test_git_source_no_tag_version() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::empty().expect("Failed to create git repository");

    let output = TestCommand::new()
        .current_dir(fixture.path())
        .args_from_str("version --source git")
        .assert_failure();

    let stderr = output.stderr();
    assert!(
        stderr.contains("Error: No version tags found in git repository"),
        "stderr should contain expected error message. Got: {stderr}"
    );
}
