// Tests for Quick Start documentation examples
// Ensures that the examples shown in README.md work as documented

use zerv::cli::flow::test_utils::expect_branch_hash;

use crate::integration_tests::flow::docs::test_utils::TestScenario;
#[test]
fn test_quick_start_documentation_examples() {
    // Test main branch (should produce clean version)
    let main_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0");

    main_scenario.assert_command("flow --source stdin", "1.0.0");

    // Test feature branch (should produce alpha with hash and post distance)
    let feature_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/new-auth")
        .checkout("feature/new-auth")
        .commit();

    let branch_feature_auth_hash = expect_branch_hash("feature/new-auth", 5, "59394");

    feature_scenario.assert_command(
        "flow --source stdin",
        &format!(
            "1.0.1-alpha.{}.post.1+feature.new.auth.1.g{{hex:7}}",
            branch_feature_auth_hash
        ),
    );

    // Test develop branch (should produce beta with stable number and post distance)
    let develop_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("develop")
        .checkout("develop")
        .commit()
        .commit()
        .commit(); // Multiple commits to get higher post distance

    develop_scenario.assert_command(
        "flow --source stdin",
        "1.0.1-beta.1.post.3+develop.3.g{hex:7}",
    );
}
