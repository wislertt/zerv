// Documentation tests for zerv version VCS Overrides command
// Tests ensure that README.md examples work as documented

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_zerv_version_vcs_overrides_documentation_examples() {
    let branch_name = "branch-name".to_string();

    let mut dirty_feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0-alpha.1.post.5.dev.123")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    // Test case 1: VCS Overrides - tag version override
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --tag-version \"v2.1.0-beta.1\"",
        "2.1.0-beta.1+branch.name.1.g{hex:7}",
    );

    // Test case 2: VCS Overrides - distance override
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --distance 42",
        "1.0.0-alpha.1.post.5.dev.123+branch.name.42.g{hex:7}",
    );

    // Test case 3: VCS Overrides - dirty state control
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --dirty",
        "1.0.0-alpha.1.post.5.dev.123+branch.name.1.g{hex:7}",
    );

    // Test case 4: VCS Overrides - branch override
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --bumped-branch \"release/42\"",
        "1.0.0-alpha.1.post.5.dev.123+release.42.1.g{hex:7}",
    );

    _ = dirty_feature_branch_scenario;
}
