use chrono::Utc;

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_zerv_version_schema_variants_documentation_examples() {
    let branch_name = "branch-name".to_string();
    let current_date = Utc::now().format("%Y.%-m.%-d").to_string();

    let mut dirty_feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0-alpha.1.post.5.dev.123")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    // Test case 1: standard-base - Clean releases only
    dirty_feature_branch_scenario = dirty_feature_branch_scenario
        .assert_command("version --source stdin --schema standard-base", "1.0.0");

    // Test case 2: standard-base-context - Clean releases with build context
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --schema standard-base-context",
        "1.0.0+branch.name.1.g{hex:7}",
    );

    // Test case 3: standard-base-prerelease - Pre-release support
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --schema standard-base-prerelease",
        "1.0.0-alpha.1",
    );

    // Test case 4
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --schema standard-base-prerelease-post-dev-context",
        "1.0.0-alpha.1.post.5.dev.123+branch.name.1.g{hex:7}",
    );

    // Test case 5: CalVer with pre-release and dev context
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "version --source stdin --schema calver-base-prerelease-post-dev-context",
        &format!(
            "{}-0.alpha.1.post.5.dev.123+branch.name.1.g{{hex:7}}",
            current_date
        ),
    );

    _ = dirty_feature_branch_scenario;
}
