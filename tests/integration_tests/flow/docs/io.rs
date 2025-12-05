// use zerv::cli::flow::test_utils::expect_branch_hash;

use crate::integration_tests::flow::docs::test_utils::TestScenario;
// use crate::util::TestCommand;

#[test]
fn test_io_documentation_examples() {
    let branch_name = "branch-name".to_string();
    // let branch_name_hash = expect_branch_hash(&branch_name, 5, "10192");
    let dirty_feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    // Test case 1
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin",
        "1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // // Test case 2
    // let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
    //     "flow --output-format zerv",
    //     "1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    // );

    _ = dirty_feature_branch_scenario;
}
