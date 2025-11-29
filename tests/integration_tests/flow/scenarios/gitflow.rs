// GitFlow workflow scenario integration tests

// Tests for GitFlow development workflows using actual git repositories
// Uses FlowIntegrationTestScenario with the same API as existing pipeline tests

use zerv::cli::flow::test_utils::{
    create_base_schema_test_cases,
    expect_branch_hash,
};
use zerv::test_info;
use zerv::test_utils::should_run_docker_tests;

use crate::flow::scenarios::FlowIntegrationTestScenario;

/// Test GitFlow development flow - exactly matching the unit test structure
/// Converted from src/cli/flow/pipeline.rs::test_gitflow_development_flow()
#[test]
fn test_gitflow_development_flow() {
    test_info!("Starting GitFlow development flow test (exactly matching Mermaid diagram)");
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Step 1: Initial state: main and develop branches
    test_info!("Step 1: Initial setup: main branch state with v1.0.0 tag");
    let scenario = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .expect_version("1.0.0", "1.0.0")
        .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

    // Step 2: Create develop branch with initial development commit
    test_info!("Step 2: Create develop branch and start development");
    let scenario = scenario
        .create_branch("develop")
        .checkout("develop")
        .commit()
        .expect_version(
            "1.0.1-beta.1.post.1+develop.1.g{hex:7}",
            "1.0.1b1.post1+develop.1.g{hex:7}",
        );

    // Step 3: Feature development from develop branch (trunk-based post mode)
    test_info!("Step 3: Create feature/auth branch from develop");
    let branch_feature_auth_hash = expect_branch_hash("feature/auth", 5, "92409");
    let scenario = scenario
        .create_branch("feature/auth")
        .checkout("feature/auth")
        .commit()
        .expect_version(
            &format!(
                "1.0.1-alpha.{}.post.2+feature.auth.2.g{{hex:7}}",
                branch_feature_auth_hash
            ),
            &format!(
                "1.0.1a{}.post2+feature.auth.2.g{{hex:7}}",
                branch_feature_auth_hash
            ),
        )
        .commit()
        .expect_version(
            &format!(
                "1.0.1-alpha.{}.post.3+feature.auth.3.g{{hex:7}}",
                branch_feature_auth_hash
            ),
            &format!(
                "1.0.1a{}.post3+feature.auth.3.g{{hex:7}}",
                branch_feature_auth_hash
            ),
        );

    // Step 4: Merge feature/auth back to develop
    test_info!("Step 4: Merge feature/auth back to develop");
    let scenario = scenario
        .checkout("develop")
        .merge_branch("feature/auth")
        .expect_version(
            "1.0.1-beta.1.post.3+develop.3.g{hex:7}",
            "1.0.1b1.post3+develop.3.g{hex:7}",
        );

    // Step 5: Hotfix emergency flow from main
    test_info!("Step 5: Create hotfix/critical branch from main for emergency fix");
    let branch_hotfix_hash = expect_branch_hash("hotfix/critical", 5, "11477");
    let scenario = scenario
        .checkout("main")
        .create_branch("hotfix/critical")
        .checkout("hotfix/critical")
        .commit()
        .expect_version(
            &format!(
                "1.0.1-alpha.{}.post.1+hotfix.critical.1.g{{hex:7}}",
                branch_hotfix_hash
            ),
            &format!(
                "1.0.1a{}.post1+hotfix.critical.1.g{{hex:7}}",
                branch_hotfix_hash
            ),
        );

    // Step 6: Merge hotfix to main and release v1.0.1
    test_info!("Step 6: Merge hotfix to main and release v1.0.1");
    let scenario = scenario
        .checkout("main")
        .merge_branch("hotfix/critical")
        .create_tag("v1.0.1")
        .expect_version("1.0.1", "1.0.1");

    // Step 7: Sync develop with main changes and continue development
    test_info!("Step 7: Sync develop with main changes");
    let scenario = scenario
        .checkout("develop")
        .merge_branch("main")
        .expect_version(
            "1.0.2-beta.1.post.4+develop.4.g{hex:7}",
            "1.0.2b1.post4+develop.4.g{hex:7}",
        );

    // Step 8: Continue development on develop
    test_info!("Step 8: Continue development on develop branch");
    let scenario = scenario.commit().expect_version(
        "1.0.2-beta.1.post.5+develop.5.g{hex:7}",
        "1.0.2b1.post5+develop.5.g{hex:7}",
    );

    // Step 9: Release branch preparation (release/1) from develop
    test_info!("Step 9: Create release/1 branch from develop for release preparation");
    let scenario = scenario
        .create_branch("release/1")
        .checkout("release/1")
        .commit()
        .expect_version(
            "1.0.2-rc.1.post.1.dev.{timestamp:now}+release.1.6.g{hex:7}",
            "1.0.2rc1.post1.dev{timestamp:now}+release.1.6.g{hex:7}",
        )
        .create_tag("1.0.2-rc.1.post.1")
        .expect_version("1.0.2-rc.1.post.1", "1.0.2rc1.post1")
        .commit()
        .expect_version(
            "1.0.2-rc.1.post.2.dev.{timestamp:now}+release.1.1.g{hex:7}",
            "1.0.2rc1.post2.dev{timestamp:now}+release.1.1.g{hex:7}",
        )
        .create_tag("1.0.2-rc.1.post.2")
        .expect_version("1.0.2-rc.1.post.2", "1.0.2rc1.post2")
        .make_dirty()
        .expect_version(
            "1.0.2-rc.1.post.3.dev.{timestamp:now}+release.1.0.g{hex:7}",
            "1.0.2rc1.post3.dev{timestamp:now}+release.1.0.g{hex:7}",
        )
        .commit()
        .expect_version(
            "1.0.2-rc.1.post.3.dev.{timestamp:now}+release.1.1.g{hex:7}",
            "1.0.2rc1.post3.dev{timestamp:now}+release.1.1.g{hex:7}",
        )
        .make_dirty()
        .expect_version(
            "1.0.2-rc.1.post.3.dev.{timestamp:now}+release.1.1.g{hex:7}",
            "1.0.2rc1.post3.dev{timestamp:now}+release.1.1.g{hex:7}",
        )
        .commit()
        .create_tag("1.0.2-rc.1.post.3")
        .expect_version("1.0.2-rc.1.post.3", "1.0.2rc1.post3");

    // Step 10: Final release merge to main
    test_info!("Step 10: Final release merge to main and release v1.1.0");
    let scenario = scenario
        .checkout("main")
        .merge_branch("release/1")
        .create_tag("v1.1.0")
        .expect_version("1.1.0", "1.1.0");

    // Step 11: Sync develop with release for next cycle
    test_info!("Step 11: Sync develop with release and prepare for next cycle");
    let scenario = scenario
        .checkout("develop")
        .merge_branch("main")
        .commit()
        .expect_version(
            "1.1.1-beta.1.post.1+develop.1.g{hex:7}",
            "1.1.1b1.post1+develop.1.g{hex:7}",
        );

    test_info!("GitFlow test completed successfully - full scenario implemented");

    let _ = scenario;
}
