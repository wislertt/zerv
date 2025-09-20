use super::{GitRepoFixture, VersionCommandUtils};
use zerv::test_utils::{VersionTestUtils, should_run_docker_tests};

// TODO: assert by output as zerv ron object and parse to zerv object back.
#[test]
fn test_version_command_in_tagged_repo() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged("v1.2.3").expect("Failed to create tagged repo");
    let output = VersionCommandUtils::run_version_command(&fixture);

    // Tier 1: Tagged, clean → major.minor.patch (exact match)
    VersionTestUtils::assert_exact_version(&output, "1.2.3", "tagged_clean");
}

#[test]
fn test_version_command_with_distance() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture =
        GitRepoFixture::with_distance("v1.0.0", 3).expect("Failed to create repo with distance");
    let output = VersionCommandUtils::run_version_command(&fixture);

    // Tier 2: Distance, clean → major.minor.patch+branch.<commit>
    VersionTestUtils::assert_version_pattern(
        &output,
        "1.0.0+main.<commit>",
        "tagged_with_distance",
    );
    VersionTestUtils::assert_version_components(&output, "1.0.0", "tagged_with_distance");
}

#[test]
fn test_version_command_dirty_repo() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::dirty("v2.1.0").expect("Failed to create dirty repo");
    let output = VersionCommandUtils::run_version_command(&fixture);

    // Tier 3: Dirty → major.minor.patch.dev<timestamp>+branch.<commit>
    // Should contain base version and additional dirty components
    VersionTestUtils::assert_version_components(&output, "2.1.0", "dirty_repo");

    // Should have dev component for dirty state
    assert!(
        output.contains(".dev") || output.contains("+main."),
        "Dirty version should contain dev component or branch info: {output}"
    );
}
