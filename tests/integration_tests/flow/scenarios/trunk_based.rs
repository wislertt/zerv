// Trunk-based workflow scenario integration tests

// Tests for trunk-based development workflows using actual git repositories
// Uses FlowIntegrationTestScenario with the same API as existing pipeline tests

use zerv::cli::flow::test_utils::{
    create_base_schema_test_cases,
    expect_branch_hash,
};
use zerv::test_utils::should_run_docker_tests;

use crate::flow::scenarios::FlowIntegrationTestScenario;

/// Test trunk-based development flow - exactly matching the unit test structure
/// Converted from src/cli/flow/pipeline.rs::test_trunk_based_development_flow()
#[test]
fn test_trunk_based_development_flow() {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Step 1: Initial commit on main with v1.0.0
    let scenario = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .expect_version("1.0.0", "1.0.0")
        .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

    // Step 2: Create parallel feature branches feature-1 and feature-2 from main
    let scenario = scenario
        .create_branch("feature-1")
        .create_branch("feature-2");

    // Capture actual hash values for validation
    let branch_feature_2_hash = expect_branch_hash("feature-2", 5, "68031");
    let branch_feature_1_hash = expect_branch_hash("feature-1", 5, "42954");

    // Step 3: feature-2: Start development with dirty state (matches Mermaid REVERSE commit)
    let scenario = scenario.checkout("feature-2").make_dirty().expect_version(
        &format!(
            "1.0.1-alpha.{}.post.0.dev.{{timestamp:now}}+feature.2.0.g{{hex:7}}",
            branch_feature_2_hash
        ),
        &format!(
            "1.0.1a{}.post0.dev{{timestamp:now}}+feature.2.0.g{{hex:7}}",
            branch_feature_2_hash
        ),
    );

    // Step 4: feature-2: Create first commit
    let scenario = scenario.commit().expect_version(
        &format!(
            "1.0.1-alpha.{}.post.1+feature.2.1.g{{hex:7}}",
            branch_feature_2_hash
        ),
        &format!(
            "1.0.1a{}.post1+feature.2.1.g{{hex:7}}",
            branch_feature_2_hash
        ),
    );

    // Step 5: feature-1: Create commits (parallel development)
    let scenario = scenario
        .checkout("feature-1")
        .commit()
        .expect_version(
            &format!(
                "1.0.1-alpha.{}.post.1+feature.1.1.g{{hex:7}}",
                branch_feature_1_hash
            ),
            &format!(
                "1.0.1a{}.post1+feature.1.1.g{{hex:7}}",
                branch_feature_1_hash
            ),
        )
        .commit()
        .expect_version(
            &format!(
                "1.0.1-alpha.{}.post.2+feature.1.2.g{{hex:7}}",
                branch_feature_1_hash
            ),
            &format!(
                "1.0.1a{}.post2+feature.1.2.g{{hex:7}}",
                branch_feature_1_hash
            ),
        );

    // Step 6: feature-1: Merge to main and release v1.0.1
    let scenario = scenario
        .checkout("main")
        .merge_branch("feature-1")
        .create_tag("v1.0.1")
        .expect_version("1.0.1", "1.0.1");

    // Return the scenario (prevents unused variable warning)
    let _ = scenario;
}
