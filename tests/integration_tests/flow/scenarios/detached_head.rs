// Detached HEAD workflow scenario integration tests

use zerv::cli::flow::test_utils::expect_branch_hash;
use zerv::test_utils::should_run_docker_tests;

use crate::flow::scenarios::FlowIntegrationTestScenario;

/// Test detached HEAD flow - when there is no branch (detached HEAD state)
#[test]
fn test_detached_head_flow() {
    if !should_run_docker_tests() {
        return;
    }

    // Create repo with history
    let scenario = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .commit()  // Commit after v1.0.0
        .commit(); // Another commit

    // Get the commit hash for v1.0.0
    let v1_hash = scenario
        .get_head_commit()
        .expect("Failed to get HEAD commit");

    // Checkout to v1.0.0 commit (this creates detached HEAD state)
    let scenario = scenario.commit().create_tag("v1.1.0");

    let scenario = scenario.checkout(&v1_hash);

    let no_branch_hash = expect_branch_hash("", 5, "34769");
    // Now we're in detached HEAD state - verify zerv handles it
    let scenario = scenario.expect_version(
        &format!("1.0.1-alpha.{}.post.2+2.g{{hex:7}}", no_branch_hash),
        &format!("1.0.1a{}.post2+2.g{{hex:7}}", no_branch_hash),
    );

    let _ = scenario;
}
