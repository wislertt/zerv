use zerv::cli::flow::test_utils::expect_branch_hash;

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

    let mut release_no_number_branch_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("release/do-something")
        .checkout("release/do-something")
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
    let branch_name_hash = expect_branch_hash("branch-name", 5, "10192");
    feature_branch_scenario = feature_branch_scenario.assert_command(
        "flow --source stdin",
        &format!(
            "1.0.1-alpha.{}.post.1+branch.name.1.g{{hex:7}}",
            branch_name_hash
        ),
    );

    // Test case 4
    let branch_name_hash = expect_branch_hash("release/do-something", 5, "48993");
    release_no_number_branch_scenario = release_no_number_branch_scenario.assert_command(
        "flow --source stdin",
        &format!(
            "1.0.1-rc.{}.post.1+release.do.something.1.g{{hex:7}}",
            branch_name_hash
        ),
    );
    _ = release_branch_scenario;
    _ = develop_branch_scenario;
    _ = feature_branch_scenario;
    _ = release_no_number_branch_scenario;

    // Custom branch rules configuration
    let custom_rules = r#"[
        (pattern: "staging", pre_release_label: rc, pre_release_num: 1, post_mode: commit),
        (pattern: "qa/*", pre_release_label: beta, post_mode: tag)
    ]"#;

    // Test case 5 - staging branch with custom rules
    let mut staging_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("staging")
        .checkout("staging")
        .commit();

    staging_scenario = staging_scenario.assert_command(
        &format!("flow --source stdin --branch-rules '{}'", custom_rules),
        "1.0.1-rc.1.post.1+staging.1.g{hex:7}",
    );

    // Test case 6 - qa/123 branch with custom rules
    let mut qa_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("qa/123")
        .checkout("qa/123")
        .commit();

    qa_scenario = qa_scenario.assert_command(
        &format!("flow --source stdin --branch-rules '{}'", custom_rules),
        "1.0.1-beta.123.post.1+qa.123.1.g{hex:7}",
    );

    // Test case 7 - feature branch falls back to default alpha behavior with custom rules
    let mut feature_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/new-feature")
        .checkout("feature/new-feature")
        .commit();

    let feature_hash = expect_branch_hash("feature/new-feature", 5, "20460");
    feature_scenario = feature_scenario.assert_command(
        &format!("flow --source stdin --branch-rules '{}'", custom_rules),
        &format!(
            "1.0.1-alpha.{}.post.1+feature.new.feature.1.g{{hex:7}}",
            feature_hash
        ),
    );

    _ = staging_scenario;
    _ = qa_scenario;
    _ = feature_scenario;
}
