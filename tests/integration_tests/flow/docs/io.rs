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

    // Test case 2
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command_contains(
        "flow --source stdin --output-format zerv",
        &[
            "schema:",
            "core:",
            "var(Major)",
            "var(Minor)",
            "var(Patch)",
            "extra_core:",
            "var(Epoch)",
            "var(PreRelease)",
            "vars:",
            "major: Some(1)",
            "minor: Some(0)",
            "patch: Some(1)",
            "pre_release:",
            "bumped_branch: Some(\"branch-name\")",
            "distance: Some(1)",
        ],
    );

    // Test case 3
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_commands(
        &[
            "flow --source stdin --output-format zerv",
            "version --source stdin --major 4 --output-format semver",
        ],
        "4.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 4
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-format pep440",
        "1.0.1a10192.post1.dev{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 5
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-format semver",
        "1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 6
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-prefix v --output-format semver",
        "v1.0.1-alpha.10192.post.1.dev.{timestamp:now}+branch.name.1.g{hex:7}",
    );

    // Test case 7
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"app:{{ major }}.{{ minor }}.{{ patch }}\"",
        "app:1.0.1",
    );

    // Test case 8
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"{{ semver_obj.docker }}\"",
        "1.0.1-alpha.10192.post.1.dev.{timestamp:now}-branch.name.1.g{hex:7}",
    );

    // Test case 9
    let dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --output-template \"{{ semver_obj.base_part }}++{{ semver_obj.pre_release_part }}++{{ semver_obj.build_part }}\"",
        "1.0.1++alpha.10192.post.1.dev.{timestamp:now}++branch.name.1.g{hex:7}",
    );

    _ = dirty_feature_branch_scenario;
}
