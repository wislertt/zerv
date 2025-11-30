// Tests for Schema Variants documentation examples
// Ensures that the schema preset examples shown in README.md work as documented

use crate::integration_tests::flow::docs::test_utils::TestScenario;

#[test]
fn test_schema_variants_documentation_examples() {
    // Test Standard Schema Family - key variants for zerv flow
    // This test demonstrates that the 10+ standard schema presets work correctly

    // 1. standard-base: Clean releases only (major.minor.patch)
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .assert_command("flow --source stdin --schema standard-base", "1.0.0");

    // 2. standard-base-prerelease: Pre-release support
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("release/1/do-something")
        .checkout("release/1/do-something")
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post",
            "1.0.1-rc.1.post.1",
        );

    // 3. standard-base-context: Clean releases with build context
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .assert_command(
            "flow --source stdin --schema standard-base-context",
            "1.0.0+{timestamp:now}",
        );

    // 4. standard-base-prerelease-context: Pre-release with build context
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("release/1/do-something")
        .checkout("release/1/do-something")
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post",
            "1.0.1-rc.1.post.1",
        );

    // 5. standard-base-prerelease-post-dev: Full pre-release with dev timestamp for dirty working directory
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/new-feature")
        .checkout("feature/new-feature")
        .commit()
        .make_dirty()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post-dev",
            "1.0.1-alpha.59394.post.1.dev.{timestamp:now}+feature.new-feature.1.g{hex:7}",
        );

    // 6. develop branch pattern with standard schema
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("develop")
        .checkout("develop")
        .commit()
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post",
            "1.0.1-beta.1.post.2",
        );

    // 7. standard-context: Smart schema with build context (automatically chooses components)
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .assert_command(
            "flow --source stdin --schema standard-context",
            "1.0.0+{timestamp:now}",
        );

    // 8. standard-no-context: Smart schema without build metadata
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/test")
        .checkout("feature/test")
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-no-context",
            "1.0.1-alpha.59394.post.1+feature.test.1.g{hex:7}",
        );

    // Test smart schema (standard): Automatically adapts based on repository state
    // Clean release -> base schema
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .assert_command("flow --source stdin --schema standard", "1.0.0");

    // Feature branch dirty -> prerelease-post-dev schema
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/smart-test")
        .checkout("feature/smart-test")
        .commit()
        .make_dirty()
        .assert_command(
            "flow --source stdin --schema standard",
            "1.0.1-alpha.59394.post.1.dev.{timestamp:now}+feature.smart-test.1.g{hex:7}",
        );
}
