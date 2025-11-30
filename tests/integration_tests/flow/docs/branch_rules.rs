use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_branch_rules_documentation_examples() {
    let mut feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("branch-name")
        .checkout("branch-name")
        .commit();

    let mut develop_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("develop")
        .checkout("develop")
        .commit();

    let mut release_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("release/1/do-something")
        .checkout("release/1/do-something")
        .commit();

    // Test case 1
    release_branch_scenario = release_branch_scenario.assert_command(
        "flow --source stdin",
        "1.0.1-rc.1.post.1+release.1.do.something.1.g{hex:7}",
    );

    // Test case 2
    develop_branch_scenario = develop_branch_scenario.assert_command(
        "flow --source stdin",
        "1.0.1-beta.1.post.1+develop.1.g{hex:7}",
    );

    // Test case 3
    feature_branch_scenario = feature_branch_scenario.assert_command(
        "flow --source stdin",
        "1.0.1-alpha.10192.post.1+branch.name.1.g{hex:7}",
    );

    _ = release_branch_scenario;
    _ = develop_branch_scenario;
    _ = feature_branch_scenario;
}
