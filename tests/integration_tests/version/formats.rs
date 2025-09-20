use super::GitRepoFixture;
use crate::integration_tests::test_version_output_format;
use rstest::rstest;
use zerv::test_utils::should_run_docker_tests;

#[rstest]
#[case("pep440", "v2.0.0", "2.0.0")]
#[case("semver", "v1.5.2", "1.5.2")]
#[case("pep440", "v1.0.0-rc.1", "1.0.0rc1")]
#[case("semver", "v2.1.0-beta.3", "2.1.0-beta.3")]
fn test_version_command_output_formats(
    #[case] format: &str,
    #[case] tag: &str,
    #[case] expected: &str,
) {
    if !should_run_docker_tests() {
        return;
    }

    let fixture = GitRepoFixture::tagged(tag).expect("Failed to create tagged repo");

    let output =
        test_version_output_format(&fixture, format).expect("Failed to test output format");

    // Should contain expected version numbers
    assert!(
        output.contains(expected),
        "Version should contain {expected} for {format}: {output}"
    );
}
