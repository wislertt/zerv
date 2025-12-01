// Tests for Pre-release Control and Post Mode Options documentation examples
// Ensures that the pre-release control examples shown in README.md work as documented

use zerv::cli::flow::test_utils::expect_branch_hash;

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_pre_release_control_and_post_mode_examples() {
    // Test explicit pre-release label and number
    let mut explicit_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("custom-branch")
        .checkout("custom-branch")
        .commit()
        .commit()
        .commit()
        .commit()
        .commit();

    explicit_scenario = explicit_scenario.assert_command(
        "flow --source stdin --pre-release-label rc --pre-release-num 3",
        "1.0.1-rc.3.post.5+custom.branch.5.g{hex:7}",
    );

    // Test extracted number with release branch
    let mut release42_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("release/42")
        .checkout("release/42")
        .commit();

    release42_scenario = release42_scenario.assert_command(
        "flow --source stdin --pre-release-label rc",
        "1.0.1-rc.42.post.1+release.42.1.g{hex:7}",
    );

    // Test post-mode differences
    let mut post_mode_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("test-branch")
        .checkout("test-branch")
        .commit()
        .commit()
        .commit();

    // Test commit mode (default)
    post_mode_scenario = post_mode_scenario.assert_command(
        "flow --source stdin --pre-release-label beta --pre-release-num 2",
        "1.0.1-beta.2.post.3+test.branch.3.g{hex:7}",
    );

    // Test hash-based identification
    let hotfix_hash = expect_branch_hash("hotfix/critical", 5, "11477");
    let mut hotfix_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("hotfix/critical")
        .checkout("hotfix/critical")
        .commit()
        .commit()
        .commit();

    hotfix_scenario = hotfix_scenario.assert_command(
        "flow --source stdin --pre-release-label alpha",
        &format!(
            "1.0.1-alpha.{}.post.3+hotfix.critical.3.g{{hex:7}}",
            hotfix_hash
        ),
    );

    // Test dirty state with manual pre-release control
    let mut dirty_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/auth")
        .checkout("feature/auth")
        .commit()
        .commit()
        .commit()
        .commit()
        .make_dirty();

    dirty_scenario = dirty_scenario.assert_command(
        "flow --source stdin --pre-release-label beta --pre-release-num 5",
        "1.0.1-beta.5.post.4.dev.{timestamp:now}+feature.auth.4.g{hex:7}",
    );

    _ = explicit_scenario;
    _ = release42_scenario;
    _ = post_mode_scenario;
    _ = hotfix_scenario;
    _ = dirty_scenario;
}
