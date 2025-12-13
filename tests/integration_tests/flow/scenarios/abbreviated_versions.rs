// Abbreviated version tag scenario integration tests

// Tests for git repositories with abbreviated version tags (v0, v0.7, etc.)
// Verifies that zerv correctly normalizes and selects the highest version

// use zerv::cli::flow::test_utils::create_base_schema_test_cases;
use zerv::test_info;
use zerv::test_utils::should_run_docker_tests;

use crate::flow::scenarios::FlowIntegrationTestScenario;

/// Test abbreviated version tags - v0, v0.7, v0.7.87 at the same commit
/// Zerv should normalize these and select 0.7.87 as the highest version
#[test]
fn test_abbreviated_version_tags_same_commit() {
    test_info!("Starting abbreviated version tags test - v0, v0.7, v0.7.87 at same commit");
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Create initial commit
    test_info!("Step 1: Create initial commit");
    let _ = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .commit()
        .create_tag("v0")
        .create_tag("v0.7")
        .create_tag("v0.7.87")
        .expect_version("0.7.87", "0.7.87");
    // .expect_schema_variants(create_base_schema_test_cases("0.7.87", "main"));

    test_info!("Successfully verified abbreviated version tag handling - zerv returns 0.7.87");
}
