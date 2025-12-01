// Tests for Schema Variants documentation examples
// Ensures that the schema preset examples shown in README.md work as documented

use zerv::cli::flow::test_utils::expect_branch_hash;

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_schema_variants_documentation_examples() {
    // Test Standard Schema Family - key variants for zerv flow
    // This test demonstrates that the 10+ standard schema presets work correctly
    let branch_name = "branch-name".to_string();
    let branch_name_hash = expect_branch_hash(&branch_name, 5, "10192");
    let mut feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit();
    let mut dirty_feature_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    let mut main_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0");

    let mut release_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_branch("release/1/do-something")
        .checkout("release/1/do-something")
        .create_tag("v1.0.1-rc.1");

    // Test case 1: standard-base - Clean releases only (major.minor.patch)
    dirty_feature_branch_scenario = dirty_feature_branch_scenario
        .assert_command("flow --source stdin --schema standard-base", "1.0.1");

    // Test case 2: standard-base-context - Clean releases with build context
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-context",
        "1.0.1+branch.name.1.g{hex:7}",
    );

    // Test case 3: standard-base-prerelease - Pre-release support
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-prerelease",
        &format!("1.0.1-alpha.{}", branch_name_hash),
    );

    // Test case 4: standard-base-prerelease-context - Pre-release with build context
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-prerelease-context",
        &format!("1.0.1-alpha.{}+branch.name.1.g{{hex:7}}", branch_name_hash),
    );

    // Test case 5
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-prerelease-post",
        &format!("1.0.1-alpha.{}.post.1", branch_name_hash),
    );

    // Test case 6
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-prerelease-post-context",
        &format!(
            "1.0.1-alpha.{}.post.1+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    // Test case 7
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-prerelease-post-dev",
        &format!(
            "1.0.1-alpha.{}.post.1.dev.{{timestamp:now}}",
            branch_name_hash
        ),
    );

    // Test case 8
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-base-prerelease-post-dev-context",
        &format!(
            "1.0.1-alpha.{}.post.1.dev.{{timestamp:now}}+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    // Test case 9
    main_branch_scenario =
        main_branch_scenario.assert_command("flow --source stdin --schema standard", "1.0.0");

    // Test case 10
    release_branch_scenario = release_branch_scenario
        .assert_command("flow --source stdin --schema standard", "1.0.1-rc.1");

    // Test case 11
    feature_branch_scenario = feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard",
        &format!(
            "1.0.1-alpha.{}.post.1+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    // Test case 12
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard",
        &format!(
            "1.0.1-alpha.{}.post.1.dev.{{timestamp:now}}+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    // Test case 13
    main_branch_scenario = main_branch_scenario
        .assert_command("flow --source stdin --schema standard-no-context", "1.0.0");

    // Test case 14
    release_branch_scenario = release_branch_scenario.assert_command(
        "flow --source stdin --schema standard-no-context",
        "1.0.1-rc.1",
    );

    // Test case 15
    feature_branch_scenario = feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-no-context",
        &format!("1.0.1-alpha.{}.post.1", branch_name_hash),
    );

    // Test case 16
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-no-context",
        &format!(
            "1.0.1-alpha.{}.post.1.dev.{{timestamp:now}}",
            branch_name_hash
        ),
    );

    // Test case 17
    main_branch_scenario = main_branch_scenario.assert_command(
        "flow --source stdin --schema standard-context",
        "1.0.0+main.g{hex:7}",
    );

    // Test case 18
    release_branch_scenario = release_branch_scenario.assert_command(
        "flow --source stdin --schema standard-context",
        "1.0.1-rc.1+release.1.do.something.g{hex:7}",
    );

    // Test case 19
    feature_branch_scenario = feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-context",
        &format!(
            "1.0.1-alpha.{}.post.1+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    // Test case 20
    dirty_feature_branch_scenario = dirty_feature_branch_scenario.assert_command(
        "flow --source stdin --schema standard-context",
        &format!(
            "1.0.1-alpha.{}.post.1.dev.{{timestamp:now}}+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    _ = main_branch_scenario;
    _ = release_branch_scenario;
    _ = feature_branch_scenario;
    _ = dirty_feature_branch_scenario;
}
