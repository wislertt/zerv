// Tests for Override Controls documentation examples
// Ensures that the override controls examples shown in README.md work as documented

use zerv::cli::flow::test_utils::expect_branch_hash;

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_individual_override_options() {
    // Test individual VCS override options with their expected outputs

    // Test --tag-version override
    let mut tag_version_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .checkout("main");

    tag_version_scenario = tag_version_scenario.assert_command(
        "flow --source stdin --tag-version \"v2.1.0-beta.1\"",
        "2.1.0",
    );

    // Test --distance override
    let feature_test_hash = expect_branch_hash("feature/test", 5, "60124");
    let mut distance_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/test")
        .checkout("feature/test")
        .commit();

    distance_scenario = distance_scenario.assert_command(
        "flow --source stdin --distance 42",
        &format!(
            "1.0.1-alpha.{}.post.42+feature.test.42.g{{hex:7}}",
            feature_test_hash
        ),
    );

    // Test --dirty override
    let feature_dirty_hash = expect_branch_hash("feature/dirty", 5, "18373");
    let mut dirty_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/dirty")
        .checkout("feature/dirty")
        .make_dirty();

    dirty_scenario = dirty_scenario.assert_command(
        "flow --source stdin --dirty",
        &format!(
            "1.0.1-alpha.{}.dev.{{timestamp:now}}+feature.dirty.g{{hex:7}}",
            feature_dirty_hash
        ),
    );

    // Test --no-dirty override
    let mut no_dirty_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/clean")
        .checkout("feature/clean")
        .make_dirty();

    no_dirty_scenario = no_dirty_scenario.assert_command(
        "flow --source stdin --no-dirty",
        "1.0.0+feature.clean.g{hex:7}",
    );

    // Test --clean override
    let mut clean_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/clean-force")
        .checkout("feature/clean-force")
        .commit()
        .make_dirty();

    clean_scenario = clean_scenario.assert_command(
        "flow --source stdin --clean",
        "1.0.0+feature.clean.force.g{hex:7}",
    );

    // Test --bumped-branch override
    let mut bumped_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("some-branch")
        .checkout("some-branch")
        .commit();

    bumped_branch_scenario = bumped_branch_scenario.assert_command(
        "flow --source stdin --bumped-branch \"release/42\"",
        "1.0.1-rc.42.post.1+release.42.1.g{hex:7}",
    );

    // Test --bumped-commit-hash override
    let feature_hash_hash = expect_branch_hash("feature/hash", 5, "48498");
    let mut bumped_hash_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/hash")
        .checkout("feature/hash")
        .commit();

    bumped_hash_scenario = bumped_hash_scenario.assert_command(
        "flow --source stdin --bumped-commit-hash \"a1b2c3d\"",
        &format!(
            "1.0.1-alpha.{}.post.1+feature.hash.1.a1b2c3d",
            feature_hash_hash
        ),
    );

    // Test --bumped-timestamp override (simplified)
    let mut bumped_timestamp_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/timestamp")
        .checkout("feature/timestamp")
        .commit()
        .make_dirty();

    bumped_timestamp_scenario = bumped_timestamp_scenario.assert_command(
        "flow --source stdin --bumped-timestamp 1729924622",
        "1.0.1-alpha.{regex:\\d+}.post.1.dev.{timestamp:now}+feature.timestamp.1.g{hex:7}",
    );

    // Test version component overrides
    let mut major_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .checkout("main");

    major_scenario = major_scenario.assert_command("flow --source stdin --major 2", "2.0.0");

    let mut minor_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .checkout("main");

    minor_scenario = minor_scenario.assert_command("flow --source stdin --minor 5", "1.5.0");

    let mut patch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .checkout("main");

    patch_scenario = patch_scenario.assert_command("flow --source stdin --patch 3", "1.0.3");

    let mut epoch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .checkout("main");

    epoch_scenario =
        epoch_scenario.assert_command("flow --source stdin --epoch 1", "1.0.0-epoch.1");

    let feature_post_hash = expect_branch_hash("feature/post", 5, "15355");
    let mut post_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/post")
        .checkout("feature/post")
        .commit();

    post_scenario = post_scenario.assert_command(
        "flow --source stdin --post 7",
        &format!(
            "1.0.1-alpha.{}.post.8+feature.post.1.g{{hex:7}}",
            feature_post_hash
        ),
    );

    // Test pre-release controls
    let feature_pr_label_hash = expect_branch_hash("feature/pr-label", 5, "10180");
    let mut pre_release_label_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/pr-label")
        .checkout("feature/pr-label")
        .commit();

    pre_release_label_scenario = pre_release_label_scenario.assert_command(
        "flow --source stdin --pre-release-label rc",
        &format!(
            "1.0.1-rc.{}.post.1+feature.pr.label.1.g{{hex:7}}",
            feature_pr_label_hash
        ),
    );

    let mut pre_release_num_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/pr-num")
        .checkout("feature/pr-num")
        .commit();

    pre_release_num_scenario = pre_release_num_scenario.assert_command(
        "flow --source stdin --pre-release-num 3",
        "1.0.1-alpha.3.post.1+feature.pr.num.1.g{hex:7}",
    );

    let feature_post_mode_hash = expect_branch_hash("feature/post-mode", 5, "17003");
    let mut post_mode_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/post-mode")
        .checkout("feature/post-mode")
        .commit();

    post_mode_scenario = post_mode_scenario.assert_command(
        "flow --source stdin --post-mode commit",
        &format!(
            "1.0.1-alpha.{}.post.1+feature.post.mode.1.g{{hex:7}}",
            feature_post_mode_hash
        ),
    );

    _ = tag_version_scenario;
    _ = distance_scenario;
    _ = dirty_scenario;
    _ = no_dirty_scenario;
    _ = clean_scenario;
    _ = bumped_branch_scenario;
    _ = bumped_hash_scenario;
    _ = bumped_timestamp_scenario;
    _ = major_scenario;
    _ = minor_scenario;
    _ = patch_scenario;
    _ = epoch_scenario;
    _ = post_scenario;
    _ = pre_release_label_scenario;
    _ = pre_release_num_scenario;
    _ = post_mode_scenario;
}

#[test]
fn test_override_controls_documentation_examples() {
    // Test complete VCS override
    let release_candidate_hash = expect_branch_hash("release/candidate", 5, "71808");
    let mut vcs_override_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("some-other-branch")
        .checkout("some-other-branch")
        .commit()
        .commit()
        .commit();

    vcs_override_scenario = vcs_override_scenario.assert_command(
        "flow --source stdin --tag-version \"v2.0.0\" --distance 5 --bumped-branch \"release/candidate\"",
        &format!(
            "2.0.1-rc.{}.post.1+release.candidate.5.g{{hex:7}}",
            release_candidate_hash
        ),
    );

    // Test version component override
    let mut version_component_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("main")
        .checkout("main");

    version_component_scenario = version_component_scenario
        .assert_command("flow --source stdin --major 1 --patch 42", "1.0.42");

    // Test mixed overrides: VCS + version components
    let mut mixed_override_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/test")
        .checkout("feature/test")
        .commit()
        .commit()
        .commit();

    mixed_override_scenario = mixed_override_scenario.assert_command(
        "flow --source stdin --distance 3 --major 2 --minor 1",
        "2.1.1-alpha.{regex:\\d+}.post.3+feature.test.3.g{hex:7}",
    );

    // Test clean release enforcement
    let mut clean_release_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("dirty-branch")
        .checkout("dirty-branch")
        .commit()
        .make_dirty();

    clean_release_scenario = clean_release_scenario.assert_command(
        "flow --source stdin --clean --major 2 --minor 0 --patch 0",
        "2.0.0+dirty.branch.g{hex:7}",
    );

    // Test dirty state control for nightly builds
    let mut nightly_build_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/nightly")
        .checkout("feature/nightly")
        .commit()
        .commit()
        .commit()
        .make_dirty();

    nightly_build_scenario = nightly_build_scenario.assert_command(
        "flow --source stdin --pre-release-label beta --pre-release-num 1",
        "1.0.1-beta.1.post.3.dev.{timestamp:now}+feature.nightly.3.g{hex:7}",
    );

    // Test complex override scenario
    let dev_branch_hash = expect_branch_hash("dev-branch", 5, "11178");
    let mut complex_override_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("dev-branch")
        .checkout("dev-branch")
        .commit()
        .commit();

    complex_override_scenario = complex_override_scenario.assert_command(
        "flow --source stdin --tag-version \"v1.5.0-rc.1\" --distance 2 --bumped-commit-hash \"f4a8b9c\" --bumped-timestamp 1729924622 --major 1 --minor 6 --post 0",
        &format!(
            "1.6.0-alpha.{}.post.2+dev.branch.2.f4a8b9c",
            dev_branch_hash
        ),
    );

    // Test template-based version component override
    let mut template_override_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("template-test")
        .checkout("template-test");

    template_override_scenario = template_override_scenario
        .assert_command("flow --source stdin --major 2 --minor 123", "2.123.0");

    // Test the comprehensive example from the breakdown
    let mut breakdown_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("actual-branch")
        .checkout("actual-branch")
        .commit()
        .commit()
        .commit()
        .commit()
        .commit()
        .commit()
        .commit();

    breakdown_scenario = breakdown_scenario.assert_command(
        "flow --source stdin --tag-version \"v2.1.0-beta.1\" --distance 7 --no-dirty --bumped-branch \"release/2\" --major 2 --minor 2 --post 0",
        "2.2.0-rc.2.post.1+release.2.7.g{hex:7}",
    );

    _ = vcs_override_scenario;
    _ = version_component_scenario;
    _ = mixed_override_scenario;
    _ = clean_release_scenario;
    _ = nightly_build_scenario;
    _ = complex_override_scenario;
    _ = template_override_scenario;
    _ = breakdown_scenario;
}
