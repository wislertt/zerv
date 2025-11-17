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
/// Only Step 1: Initial setup on main with v1.0.0
#[test]
fn test_trunk_based_development_flow() {
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Step 1: Initial commit on main with v1.0.0
    let _scenario = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .expect_version("1.0.0", "1.0.0")
        .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

    // Demonstrate usage of expect_branch_hash from unit test utilities
    // This validates branch hash generation consistency between unit and integration tests
    let _main_hash = expect_branch_hash("main", 7, "1446771");
}
