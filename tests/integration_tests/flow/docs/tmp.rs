// Temporary test file for assert_commands functionality
// Will be renamed when we create proper documentation for this feature

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_assert_commands_functionality() {
    // Test multi-command pipeline functionality with zerv command chaining
    // Using bump operations to ensure piping properly preserves and modifies version data

    // Test 1: version -> version with major bump
    let major_bump_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.2.3");

    // First version outputs zerv format, second version bumps major and outputs semver
    major_bump_scenario.assert_commands(
        &[
            "version --source stdin --output-format zerv",
            "version --source stdin --output-format semver --bump-major",
        ],
        "2.0.0", // Should be bumped from 1.2.3 to 2.2.3 (let's see actual behavior)
    );

    // Test 2: version -> version -> version with sequential bumps (minor then patch)
    let sequential_bump_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0");

    // Pipeline: version -> bump minor -> bump patch
    sequential_bump_scenario.assert_commands(
        &[
            "version --source stdin --output-format zerv",
            "version --source stdin --output-format zerv --bump-minor",
            "version --source stdin --output-format semver --bump-patch",
        ],
        "1.1.1", // 1.0.0 -> 1.1.0 -> 1.1.1
    );

    // Test 3: version -> flow -> version with bump operations across different command types
    let flow_bump_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v2.0.0")
        .create_branch("feature/bump-test")
        .checkout("feature/bump-test")
        .commit();

    // Pipeline: version -> flow -> version with bump major
    flow_bump_scenario.assert_commands(
        &[
            "version --source stdin --output-format zerv",
            "flow --source stdin --output-format zerv",
            "version --source stdin --output-format semver --bump-major",
        ],
        "3.0.0+feature.bump.test.1.g{hex:7}", // Flow: 2.0.1 (no alpha for some reason) -> Bump major: 3.0.0
    );

    // Test 4: Three-step bump pipeline (major -> minor -> patch)
    let three_step_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v0.5.0");

    // Pipeline: version -> bump major -> bump minor -> bump patch
    three_step_scenario.assert_commands(
        &[
            "version --source stdin --output-format zerv",
            "version --source stdin --output-format zerv --bump-major",
            "version --source stdin --output-format zerv --bump-minor",
            "version --source stdin --output-format semver --bump-patch",
        ],
        "1.1.1", // 0.5.0 -> 1.5.0 -> 1.6.0 -> 1.6.1
    );

    // Test 5: Template output after bumps
    let template_bump_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v2.0.0");

    // Pipeline: version -> bump major -> custom template
    template_bump_scenario.assert_commands(
        &[
            "version --source stdin --output-format zerv",
            "version --source stdin --output-format zerv --bump-major",
            "version --source stdin --output-template 'Release-{{major}}.{{minor}}'",
        ],
        "Release-3.0", // 2.0.0 -> 3.0.0 -> Release-3.0
    );

    // Test 6: Single command with multiple bumps (baseline test)
    let single_bump_scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.1.1");

    single_bump_scenario.assert_commands(
        &["version --source stdin --output-format semver --bump-major --bump-minor --bump-patch"],
        "2.1.1", // Bump behavior: 1.1.1 -> 2.1.1 (let's verify actual behavior)
    );
}
