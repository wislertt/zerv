// Documentation tests for zerv version Component Overrides command
// Tests ensure that README.md examples work as documented

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_zerv_version_component_overrides_documentation_examples() {
    let branch_name = "branch-name".to_string();

    let scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch(&branch_name)
        .checkout(&branch_name)
        .commit()
        .make_dirty();

    // Test case 1: Version component overrides (major, minor, patch)
    let scenario = scenario.assert_command(
        "version --source stdin --major 2 --minor 5 --patch 99",
        "2.5.99+branch.name.1.g{hex:7}",
    );

    // Test case 2: Pre-release component overrides (label and number)
    let scenario = scenario.assert_command(
        "version --source stdin --pre-release-label rc --pre-release-num 3",
        "1.0.0-rc.3+branch.name.1.g{hex:7}",
    );

    // Test case 3: Additional component overrides (epoch, post, dev)
    let scenario = scenario.assert_command(
        "version --source stdin --epoch 1 --post 7 --dev 456",
        "1.0.0-epoch.1.post.7.dev.456+branch.name.1.g{hex:7}",
    );

    // Test case 4: Custom variables override with schema-ron
    let scenario = scenario.assert_command(
        "version --source stdin --schema-ron '(core:[var(Major), var(Minor), var(Patch)], extra_core:[], build:[var(custom(\"build_id\")), var(custom(\"environment\"))])' --custom '{\"build_id\": \"prod-123\", \"environment\": \"staging\"}'",
        "1.0.0+prod.123.staging",
    );

    _ = scenario;
}
