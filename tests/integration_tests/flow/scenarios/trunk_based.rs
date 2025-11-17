// Trunk-based workflow scenario integration tests

// Tests for trunk-based development workflows using actual git repositories
// Uses FlowIntegrationTestScenario with the same API as existing pipeline tests

use zerv::cli::flow::test_utils::{
    SchemaTestBuild,
    SchemaTestExtraCore,
    create_base_schema_test_cases,
    create_full_schema_test_cases,
    expect_branch_hash,
};
use zerv::test_info;
use zerv::test_utils::should_run_docker_tests;
use zerv::version::zerv::PreReleaseLabel;

use crate::flow::scenarios::FlowIntegrationTestScenario;

/// Test trunk-based development flow - exactly matching the unit test structure
/// Converted from src/cli/flow/pipeline.rs::test_trunk_based_development_flow()
#[test]
fn test_trunk_based_development_flow() {
    test_info!("Starting trunk-based development flow test (exactly matching Mermaid diagram)");
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Step 1: Initial commit on main with v1.0.0
    test_info!("Step 1: Initial setup: main branch state with v1.0.0 tag");
    let scenario = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .expect_version("1.0.0", "1.0.0")
        .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

    // Step 2: Create parallel feature branches feature-1 and feature-2 from main
    test_info!("Step 2: Create parallel feature branches: feature-1 and feature-2");
    let scenario = scenario
        .create_branch("feature-1")
        .create_branch("feature-2");

    // Capture actual hash values for validation
    let branch_feature_2_hash = expect_branch_hash("feature-2", 5, "68031");
    let branch_feature_1_hash = expect_branch_hash("feature-1", 5, "42954");

    // Step 3: feature-2: Start development with dirty state (matches Mermaid REVERSE commit)
    test_info!("Step 3: feature-2: Start development with dirty state");
    let scenario = scenario
        .checkout("feature-2")
        .make_dirty()
        .expect_version(
            &format!(
                "1.0.1-alpha.{}.post.0.dev.{{timestamp:now}}+feature.2.0.g{{hex:7}}",
                branch_feature_2_hash
            ),
            &format!(
                "1.0.1a{}.post0.dev{{timestamp:now}}+feature.2.0.g{{hex:7}}",
                branch_feature_2_hash
            ),
        )
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Alpha,
                pre_release_num: &branch_feature_2_hash.to_string(),
                post: 0,
                dev: Some("{timestamp:now}"),
            },
            SchemaTestBuild {
                sanitized_branch_name: "feature.2",
                distance: 0,
                include_build_for_standard: true,
            },
        ));

    // Step 4: feature-2: Create first commit
    test_info!("Step 4: feature-2: Create first commit");
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
    test_info!("Step 5: feature-1: Create commits (parallel development)");
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
    test_info!("Step 6: feature-1: Merge to main and release v1.0.1");
    let scenario = scenario
        .checkout("main")
        .merge_branch("feature-1")
        .create_tag("v1.0.1")
        .expect_version("1.0.1", "1.0.1");

    // Step 7: feature-2: Sync with main to get feature-1 changes
    test_info!("Step 7: feature-2: Sync with main to get feature-1 changes");
    let scenario = scenario
        .checkout("feature-2")
        .merge_branch("main")
        .expect_version(
            &format!(
                "1.0.2-alpha.{}.post.2+feature.2.2.g{{hex:7}}",
                branch_feature_2_hash
            ),
            &format!(
                "1.0.2a{}.post2+feature.2.2.g{{hex:7}}",
                branch_feature_2_hash
            ),
        );

    // Step 8: feature-2: Create additional commit
    test_info!("Step 8: feature-2: Create additional commit");
    let scenario = scenario.commit().expect_version(
        &format!(
            "1.0.2-alpha.{}.post.3+feature.2.3.g{{hex:7}}",
            branch_feature_2_hash
        ),
        &format!(
            "1.0.2a{}.post3+feature.2.3.g{{hex:7}}",
            branch_feature_2_hash
        ),
    );

    // Step 9: feature-3: Branch from feature-2 for sub-feature development
    test_info!("Step 9: feature-3: Branch from feature-2 for sub-feature development");
    let branch_feature_3_hash = expect_branch_hash("feature-3", 5, "14698");
    let scenario = scenario
        .create_branch("feature-3")
        .checkout("feature-3")
        .commit()
        .expect_version(
            &format!(
                "1.0.2-alpha.{}.post.4+feature.3.4.g{{hex:7}}",
                branch_feature_3_hash
            ),
            &format!(
                "1.0.2a{}.post4+feature.3.4.g{{hex:7}}",
                branch_feature_3_hash
            ),
        );

    // Step 10: feature-3: Continue development with dirty state
    test_info!("Step 10: feature-3: Continue development with dirty state");
    let scenario = scenario.make_dirty().expect_version(
        &format!(
            "1.0.2-alpha.{}.post.4.dev.{{timestamp:now}}+feature.3.4.g{{hex:7}}",
            branch_feature_3_hash
        ),
        &format!(
            "1.0.2a{}.post4.dev{{timestamp:now}}+feature.3.4.g{{hex:7}}",
            branch_feature_3_hash
        ),
    );

    // Step 11: feature-3: Continue development with commits
    test_info!("Step 11: feature-3: Continue development with commits");
    let scenario = scenario
        .commit()
        .expect_version(
            &format!(
                "1.0.2-alpha.{}.post.5+feature.3.5.g{{hex:7}}",
                branch_feature_3_hash
            ),
            &format!(
                "1.0.2a{}.post5+feature.3.5.g{{hex:7}}",
                branch_feature_3_hash
            ),
        )
        .commit()
        .expect_version(
            &format!(
                "1.0.2-alpha.{}.post.6+feature.3.6.g{{hex:7}}",
                branch_feature_3_hash
            ),
            &format!(
                "1.0.2a{}.post6+feature.3.6.g{{hex:7}}",
                branch_feature_3_hash
            ),
        );

    // Step 12: feature-2: Merge feature-3 back to continue development
    test_info!("Step 12: feature-2: Merge feature-3 back to continue development");
    let scenario = scenario
        .checkout("feature-2")
        .merge_branch("feature-3")
        .expect_version(
            &format!(
                "1.0.2-alpha.{}.post.6+feature.2.6.g{{hex:7}}",
                branch_feature_2_hash
            ),
            &format!(
                "1.0.2a{}.post6+feature.2.6.g{{hex:7}}",
                branch_feature_2_hash
            ),
        );

    // Step 13: feature-2: Final development before release
    test_info!("Step 13: feature-2: Final development before release");
    let scenario = scenario.commit().expect_version(
        &format!(
            "1.0.2-alpha.{}.post.7+feature.2.7.g{{hex:7}}",
            branch_feature_2_hash
        ),
        &format!(
            "1.0.2a{}.post7+feature.2.7.g{{hex:7}}",
            branch_feature_2_hash
        ),
    );

    // Step 14: Final release: feature-2 merges to main and releases v1.1.0
    test_info!("Step 14: Final release: feature-2 merges to main and releases v1.1.0");
    let scenario = scenario
        .checkout("main")
        .merge_branch("feature-2")
        .create_tag("v1.1.0")
        .expect_version("1.1.0", "1.1.0");

    test_info!("Trunk-based development flow test completed successfully!");

    // Return the scenario (prevents unused variable warning)
    let _ = scenario;
}
