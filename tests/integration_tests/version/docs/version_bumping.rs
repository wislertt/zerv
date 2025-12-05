// Documentation tests for zerv version Version Bumping command
// Tests ensure that README.md examples work as documented

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_zerv_version_version_bumping_documentation_examples() {
    let scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0");
    // Test case 1: Version Bumping - major bump
    let scenario = scenario.assert_command("version --source stdin --bump-major", "2.0.0");

    // Test case 2: Version Bumping - minor bump
    let scenario = scenario.assert_command("version --source stdin --bump-minor", "1.1.0");

    // Test case 3: Version Bumping - patch bump
    let scenario = scenario.assert_command("version --source stdin --bump-patch", "1.0.1");

    // Test case 4: Version Bumping - multiple field bumps
    let scenario =
        scenario.assert_command("version --source stdin --bump-major --bump-minor", "2.1.0");

    // Test case 5: Version Bumping - core field bump by index
    let scenario = scenario.assert_command("version --source stdin --bump-core 0", "2.0.0");

    // Test case 6: Version Bumping - multiple field bumps with patch
    let scenario = scenario.assert_command(
        "version --source stdin --bump-major --bump-minor --bump-patch",
        "2.1.1",
    );

    // Test case 7
    let scenario = scenario.assert_command("version --source stdin --bump-major 2", "3.0.0");

    _ = scenario;
}
