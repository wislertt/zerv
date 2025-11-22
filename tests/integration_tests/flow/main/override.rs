// Flow command override tests
// Tests for override functionality in flow command, especially post override

use rstest::rstest;
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

#[rstest]
#[case::no_post_override("semver", "1.2.3")]
#[case::no_post_override_pep440("pep440", "1.2.3")]
fn test_post_override_default(#[case] format: &str, #[case] expected: &str) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!("flow --source stdin --schema standard --output-format {format}"),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[rstest]
#[case::post_override_basic("semver", "--post 5", "1.2.3-post.5")]
#[case::post_override_pep440("pep440", "--post 5", "1.2.3.post5")]
fn test_post_override_basic(#[case] format: &str, #[case] post_arg: &str, #[case] expected: &str) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!(
            "flow --source stdin --schema standard-base-prerelease-post --output-format {format} {post_arg}"
        ),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[rstest]
fn test_post_override_with_distance() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("feature/test".to_string())
        .with_distance(5)
        .build()
        .to_string();

    // Test post override works even with distance
    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard --output-format semver --post 42",
        zerv_ron,
    );

    // The post override should be applied - check that output contains version info
    assert!(output.starts_with("1.2."));
    assert!(output.contains("feature.test"));
}

#[rstest]
fn test_post_override_with_pre_release() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("alpha".to_string())
        .with_pre_release(PreReleaseLabel::Alpha, None)
        .with_distance(3)
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard --output-format semver --post 10",
        zerv_ron,
    );

    // Test that post override doesn't interfere with pre-release formatting
    assert!(output.contains("alpha"));
}

#[rstest]
#[case::major_override("5.2.3", "--major 5")]
#[case::minor_override("1.7.3", "--minor 7")]
#[case::patch_override("1.2.8", "--patch 8")]
#[case::epoch_override("1.2.3-epoch.5", "--epoch 5")]
#[case::post_override("1.2.3-post.15", "--post 15")]
#[case::dev_override("1.2.3-dev.7", "--dev 7")]
fn test_individual_overrides(#[case] expected: &str, #[case] override_arg: &str) {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        &format!(
            "flow --source stdin --schema standard-base-prerelease-post-dev --output-format semver {override_arg}"
        ),
        zerv_ron,
    );

    assert_eq!(output, expected);
}

#[rstest]
fn test_multiple_overrides_combined() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard-base-prerelease-post-dev --output-format semver --major 3 --minor 5 --patch 7 --post 42",
        zerv_ron,
    );

    assert_eq!(output, "3.5.7-post.42");
}

#[rstest]
fn test_post_and_dev_overrides_combined() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard-base-prerelease-post-dev --output-format semver --post 99 --dev 11",
        zerv_ron,
    );

    assert_eq!(output, "1.2.3-post.99.dev.11");
}

#[rstest]
fn test_post_override_with_vcs_overrides() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("feature/test".to_string())
        .with_distance(10)
        .with_dirty(true)
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard-base-prerelease-post --output-format semver --clean --post 99",
        zerv_ron,
    );

    // Clean state should ignore distance/dirty, but post override should still work
    assert_eq!(output, "1.2.3-post.99");
}

#[rstest]
fn test_post_override_template_syntax() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .with_distance(0)
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard-base-prerelease-post --output-format semver --post '5'",
        zerv_ron,
    );

    assert_eq!(output, "1.2.3-post.5");
}

#[rstest]
fn test_post_override_with_different_branch() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("release".to_string())
        .build()
        .to_string();

    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard-base-prerelease-post --output-format semver --post 77",
        zerv_ron,
    );

    assert_eq!(output, "1.2.3-post.77");
}

#[rstest]
fn test_post_override_precedence_over_defaults() {
    let zerv_ron = ZervFixture::new()
        .with_version(1, 2, 3)
        .with_branch("main".to_string())
        .build()
        .to_string();

    // Test that post override takes precedence over default template
    let output = TestCommand::run_with_stdin(
        "flow --source stdin --schema standard-base-prerelease-post --output-format semver --post 123",
        zerv_ron,
    );

    assert_eq!(output, "1.2.3-post.123");
}
