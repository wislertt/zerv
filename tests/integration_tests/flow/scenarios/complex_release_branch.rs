// Complex release branch management scenario integration tests

// Tests for complex release branch scenarios including branch abandonment
// and cascading release preparation using actual git repositories
// Uses FlowIntegrationTestScenario with the same API as unit tests

use zerv::cli::flow::test_utils::{
    SchemaTestBuild,
    SchemaTestExtraCore,
    create_base_schema_test_cases,
    create_full_schema_test_cases,
};
use zerv::test_info;
use zerv::test_utils::should_run_docker_tests;
use zerv::version::zerv::PreReleaseLabel;

use crate::flow::scenarios::FlowIntegrationTestScenario;

/// Test complex release branch abandonment scenario - exactly matching the unit test structure
/// Converted from src/cli/flow/pipeline.rs::test_complex_release_branch_abandonment()
#[test]
fn test_complex_release_branch_abandonment() {
    test_info!("Starting complex release branch abandonment test");
    if !should_run_docker_tests() {
        return; // Skip when `ZERV_TEST_DOCKER` are disabled
    }

    // Step 1: Initial state: main branch with v1.0.0
    test_info!("Step 1: Initial setup: main branch state with v1.0.0 tag");
    let scenario = FlowIntegrationTestScenario::new()
        .expect("Failed to create test scenario")
        .create_tag("v1.0.0")
        .expect_version("1.0.0", "1.0.0")
        .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

    // Step 2: Create release/1 from main for next release preparation
    test_info!("Step 2: Create release/1 from main for next release preparation");
    let scenario = scenario
        .create_branch("release/1")
        .checkout("release/1")
        .commit()
        .expect_version(
            "1.0.1-rc.1.post.1+release.1.1.g{hex:7}",
            "1.0.1rc1.post1+release.1.1.g{hex:7}",
        )
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "1",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.1",
                distance: 1,
                include_build_for_standard: true,
            },
        ))
        .create_tag("v1.0.1-rc.1.post.1")
        .expect_version("1.0.1-rc.1.post.1", "1.0.1rc1.post1")
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "1",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.1",
                distance: 0,
                include_build_for_standard: false,
            },
        ))
        .commit()
        .expect_version(
            "1.0.1-rc.1.post.1+release.1.1.g{hex:7}",
            "1.0.1rc1.post1+release.1.1.g{hex:7}",
        )
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "1",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.1",
                distance: 1,
                include_build_for_standard: true,
            },
        ))
        .create_tag("v1.0.1-rc.1.post.2")
        .expect_version("1.0.1-rc.1.post.2", "1.0.1rc1.post2");

    // Step 3: Create release/2 from the second commit of release/1 (before issues)
    test_info!("Step 3: Create release/2 from second commit of release/1");
    let scenario = scenario
        .create_branch("release/2")
        .checkout("release/2")
        .commit()
        .expect_version(
            "1.0.1-rc.2.post.1+release.2.1.g{hex:7}",
            "1.0.1rc2.post1+release.2.1.g{hex:7}",
        )
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "2",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.2",
                distance: 1,
                include_build_for_standard: true,
            },
        ))
        .create_tag("v1.0.1-rc.2.post.1")
        .expect_version("1.0.1-rc.2.post.1", "1.0.1rc2.post1")
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "2",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.2",
                distance: 0,
                include_build_for_standard: false,
            },
        ));

    // Step 4: Go back to release/1 and add the problematic third commit (issues found)
    test_info!("Step 4: release/1 gets third commit with issues");
    let scenario = scenario
        .checkout("release/1")
        .expect_version("1.0.1-rc.1.post.2", "1.0.1rc1.post2")
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "1",
                post: 2,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.1",
                distance: 0,
                include_build_for_standard: false,
            },
        ))
        .commit()
        .expect_version(
            "1.0.1-rc.1.post.1+release.1.1.g{hex:7}",
            "1.0.1rc1.post1+release.1.1.g{hex:7}",
        )
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "1",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.1",
                distance: 1,
                include_build_for_standard: true,
            },
        ))
        .create_tag("v1.0.1-rc.1.post.3")
        .expect_version("1.0.1-rc.1.post.3", "1.0.1rc1.post3");

    // Step 5: release/2 completes preparation successfully
    test_info!("Step 5: release/2 completes preparation successfully");
    let scenario = scenario
        .checkout("release/2")
        .expect_version("1.0.1-rc.2.post.1", "1.0.1rc2.post1")
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "2",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.2",
                distance: 0,
                include_build_for_standard: false,
            },
        ))
        .commit()
        .expect_version(
            "1.0.1-rc.2.post.1+release.2.1.g{hex:7}",
            "1.0.1rc2.post1+release.2.1.g{hex:7}",
        )
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "2",
                post: 1,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.2",
                distance: 1,
                include_build_for_standard: true,
            },
        ))
        .create_tag("v1.0.1-rc.2.post.2")
        .expect_version("1.0.1-rc.2.post.2", "1.0.1rc2.post2");

    // Step 6: Merge release/2 to main and release v1.1.0
    test_info!("Step 6: Merge release/2 to main and release v1.1.0");
    let scenario = scenario
        .checkout("main")
        .merge_branch("release/2")
        .create_tag("v1.1.0")
        .expect_version("1.1.0", "1.1.0")
        .expect_schema_variants(create_base_schema_test_cases("1.1.0", "main"));

    // Verify release/1 remains abandoned (never merged)
    test_info!("Step 7: Verify release/1 remains abandoned");
    let scenario = scenario
        .checkout("release/1")
        .expect_version("1.0.1-rc.1.post.3", "1.0.1rc1.post3")
        .expect_schema_variants(create_full_schema_test_cases(
            "1.0.1",
            SchemaTestExtraCore {
                pre_release_label: PreReleaseLabel::Rc,
                pre_release_num: "1",
                post: 3,
                dev: None,
            },
            SchemaTestBuild {
                sanitized_branch_name: "release.1",
                distance: 0,
                include_build_for_standard: false,
            },
        ));

    test_info!("Complex release branch abandonment test completed successfully");

    // Test completes successfully - drop scenario
    drop(scenario);
}
