// Tests for Quick Start documentation examples
// Ensures that the examples shown in README.md work as documented

use zerv::cli::flow::test_utils::expect_branch_hash;

use crate::integration_tests::flow::docs::test_utils::TestScenario;
#[test]
fn test_quick_start_documentation_examples() {
    // Test main branch (should produce clean version)
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .assert_command("flow --source stdin", "1.0.0");

    // Test release branch with pre-release tag (should preserve pre-release information)
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_branch("release/1/do-something")
        .checkout("release/1/do-something")
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post",
            "1.0.1-rc.1.post.1",
        )
        .create_tag("v1.0.1-rc.1.post.1")
        .assert_command("flow --source stdin", "1.0.1-rc.1.post.1")
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post",
            "1.0.1-rc.1.post.2",
        )
        .create_tag("v1.0.1-rc.1.post.2")
        .assert_command("flow --source stdin", "1.0.1-rc.1.post.2")
        .commit()
        .assert_command(
            "flow --source stdin --schema standard-base-prerelease-post",
            "1.0.1-rc.1.post.3",
        )
        .create_tag("v1.0.1-rc.1.post.3")
        .assert_command("flow --source stdin", "1.0.1-rc.1.post.3");

    // Test develop branch (should produce beta with stable number and post distance)
    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("develop")
        .checkout("develop")
        .commit()
        .commit()
        .commit() // Multiple commits to get higher post distance
        .assert_command(
            "flow --source stdin",
            "1.0.1-beta.1.post.3+develop.3.g{hex:7}",
        );

    // Test feature branch (should produce alpha with hash and post distance)
    let branch_feature_auth_hash = expect_branch_hash("feature/new-auth", 5, "59394");

    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/new-auth")
        .checkout("feature/new-auth")
        .commit()
        .assert_command(
            "flow --source stdin",
            &format!(
                "1.0.1-alpha.{}.post.1+feature.new.auth.1.g{{hex:7}}",
                branch_feature_auth_hash
            ),
        );

    // Test dirty feature branch (should include dev timestamp)
    let branch_dirty_work_hash = expect_branch_hash("feature/dirty-work", 5, "17015");

    TestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .create_branch("feature/dirty-work")
        .checkout("feature/dirty-work")
        .commit()
        .make_dirty() // Make working directory dirty
        .assert_command(
            "flow --source stdin",
            &format!(
                "1.0.1-alpha.{}.post.1.dev.{{timestamp:now}}+feature.dirty.work.1.g{{hex:7}}",
                branch_dirty_work_hash
            ),
        );
}
