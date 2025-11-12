use ron::from_str;

use crate::cli::flow::args::FlowArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::version::pipeline::run_version_pipeline;
use crate::error::ZervError;
use crate::version::zerv::core::Zerv;

pub fn run_flow_pipeline(args: FlowArgs) -> Result<String, ZervError> {
    tracing::debug!("Starting flow pipeline with args: {:?}", args);

    let mut args = args;

    // Step 1: Get current state (no bumps)
    let current_zerv = args.get_current_zerv_object()?;

    // Step 2: Validate and apply branch rules using current state
    args.validate(&current_zerv)?;

    // Step 3: Create bumped version args
    let version_args = args.create_bumped_version_args(&current_zerv)?;

    // Step 4: Run version pipeline
    let ron_output = run_version_pipeline(version_args)?;

    let zerv_object: Zerv = from_str(&ron_output)
        .map_err(|e| ZervError::InvalidFormat(format!("Failed to parse version output: {}", e)))?;

    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output.output_format,
        args.output.output_prefix.as_deref(),
        &args.output.output_template,
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::cli::flow::test_utils::{
        FlowTestScenario,
        create_base_schema_test_cases,
        create_full_schema_test_cases,
        expect_branch_hash,
    };
    use crate::test_info;
    use crate::test_utils::should_run_docker_tests;
    use crate::version::zerv::PreReleaseLabel;

    #[test]
    fn test_trunk_based_development_flow() {
        test_info!("Starting trunk-based development flow test (exactly matching Mermaid diagram)");
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        // Step 1: Initial commit on main with v1.0.0
        test_info!("Step 1: Initial setup: main branch state with v1.0.0 tag");
        let scenario = FlowTestScenario::new()
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
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                0,
                Some("{timestamp:now}"),
                "feature.2",
                0,
            ));

        // Step 4: feature-2: Create first commit
        test_info!("Step 4: feature-2: Create first commit");
        let scenario = scenario
            .commit()
            .expect_version(
                &format!(
                    "1.0.1-alpha.{}.post.1+feature.2.1.g{{hex:7}}",
                    branch_feature_2_hash
                ),
                &format!(
                    "1.0.1a{}.post1+feature.2.1.g{{hex:7}}",
                    branch_feature_2_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                1,
                None,
                "feature.2",
                1,
            ));

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
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_feature_1_hash.to_string(),
                1,
                None,
                "feature.1",
                1,
            ))
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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_feature_1_hash.to_string(),
                2,
                None,
                "feature.1",
                2,
            ));

        // Step 6: feature-1: Merge to main and release v1.0.1
        test_info!("Step 6: feature-1: Merge to main and release v1.0.1");
        let scenario = scenario
            .checkout("main")
            .merge_branch("feature-1")
            .create_tag("v1.0.1")
            .expect_version("1.0.1", "1.0.1")
            .expect_schema_variants(create_base_schema_test_cases("1.0.1", "main"));

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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                2,
                None,
                "feature.2",
                2,
            ));

        // Step 8: feature-2: Create additional commit
        test_info!("Step 8: feature-2: Create additional commit");
        let scenario = scenario
            .commit()
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.3+feature.2.3.g{{hex:7}}",
                    branch_feature_2_hash
                ),
                &format!(
                    "1.0.2a{}.post3+feature.2.3.g{{hex:7}}",
                    branch_feature_2_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                3,
                None,
                "feature.2",
                3,
            ));

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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                4,
                None,
                "feature.3",
                4,
            ));

        // Step 10: feature-3: Continue development with dirty state
        test_info!("Step 10: feature-3: Continue development with dirty state");
        let scenario = scenario
            .make_dirty()
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.4.dev.{{timestamp:now}}+feature.3.4.g{{hex:7}}",
                    branch_feature_3_hash
                ),
                &format!(
                    "1.0.2a{}.post4.dev{{timestamp:now}}+feature.3.4.g{{hex:7}}",
                    branch_feature_3_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                4,
                Some("{timestamp:now}"),
                "feature.3",
                4,
            ));

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
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                5,
                None,
                "feature.3",
                5,
            ))
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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                6,
                None,
                "feature.3",
                6,
            ));

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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                6,
                None,
                "feature.2",
                6,
            ));

        // Step 13: feature-2: Final development before release
        test_info!("Step 13: feature-2: Final development before release");
        let scenario = scenario
            .commit()
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.7+feature.2.7.g{{hex:7}}",
                    branch_feature_2_hash
                ),
                &format!(
                    "1.0.2a{}.post7+feature.2.7.g{{hex:7}}",
                    branch_feature_2_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                7,
                None,
                "feature.2",
                7,
            ));

        // Step 14: Final release: feature-2 merges to main and releases v1.1.0
        test_info!("Step 14: Final release: feature-2 merges to main and releases v1.1.0");
        scenario
            .checkout("main")
            .merge_branch("feature-2")
            .create_tag("v1.1.0")
            .expect_version("1.1.0", "1.1.0")
            .expect_schema_variants(create_base_schema_test_cases("1.1.0", "main"));
    }

    #[test]
    fn test_gitflow_development_flow() {
        test_info!("Starting GitFlow development flow test (exactly matching Mermaid diagram)");
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        // Step 1: Initial state: main and develop branches
        test_info!("Step 1: Initial setup: main branch state with v1.0.0 tag");
        let scenario = FlowTestScenario::new()
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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Beta,
                "1", // develop uses pre_release_num: 1
                1,
                None,
                "develop",
                1,
            ));

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
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_feature_auth_hash.to_string(),
                2,
                None,
                "feature.auth",
                2,
            ))
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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_feature_auth_hash.to_string(),
                3,
                None,
                "feature.auth",
                3,
            ));

        // Step 4: Merge feature/auth back to develop
        test_info!("Step 4: Merge feature/auth back to develop");
        let scenario = scenario
            .checkout("develop")
            .merge_branch("feature/auth")
            .expect_version(
                "1.0.1-beta.1.post.3+develop.3.g{hex:7}",
                "1.0.1b1.post3+develop.3.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Beta,
                "1",
                3,
                None,
                "develop",
                3,
            ));

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
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_hotfix_hash.to_string(),
                1,
                None,
                "hotfix.critical",
                1,
            ));

        // Step 6: Merge hotfix to main and release v1.0.1
        test_info!("Step 6: Merge hotfix to main and release v1.0.1");
        let scenario = scenario
            .checkout("main")
            .merge_branch("hotfix/critical")
            .create_tag("v1.0.1")
            .expect_version("1.0.1", "1.0.1")
            .expect_schema_variants(create_base_schema_test_cases("1.0.1", "main"));

        // Step 7: Sync develop with main changes and continue development
        test_info!("Step 7: Sync develop with main changes");
        let scenario = scenario
            .checkout("develop")
            .merge_branch("main")
            .expect_version(
                "1.0.2-beta.1.post.4+develop.4.g{hex:7}",
                "1.0.2b1.post4+develop.4.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Beta,
                "1",
                4,
                None,
                "develop",
                4,
            ));

        // Step 8: Continue development on develop
        test_info!("Step 8: Continue development on develop branch");
        let scenario = scenario
            .commit()
            .expect_version(
                "1.0.2-beta.1.post.5+develop.5.g{hex:7}",
                "1.0.2b1.post5+develop.5.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Beta,
                "1",
                5,
                None,
                "develop",
                5,
            ));

        // Step 9: Release branch preparation (release/1) from develop
        test_info!("Step 9: Create release/1 branch from develop for release preparation");
        let scenario = scenario
            .create_branch("release/1")
            .checkout("release/1")
            .commit()
            .expect_version(
                "1.0.2-rc.1.post.1+release.1.6.g{hex:7}",
                "1.0.2rc1.post1+release.1.6.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Rc,
                "1", // release/1 uses pre_release_num: 1
                1,
                None,
                "release.1",
                6,
            ))
            .create_tag("v1.0.2-rc.1.post.1")
            .expect_version("1.0.2-rc.1.post.1", "1.0.2rc1.post1")
            // .expect_schema_variants(create_base_schema_test_cases("1.0.2rc1.post1", "release/1"))
            .commit()
            .expect_version(
                "1.0.2-rc.1.post.1+release.1.1.g{hex:7}",
                "1.0.2rc1.post1+release.1.1.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Rc,
                "1",
                1,
                None,
                "release.1",
                1,
            ))
            .create_tag("v1.0.2-rc.1.post.2")
            .expect_version("1.0.2-rc.1.post.2", "1.0.2rc1.post2");
        // .expect_schema_variants(create_base_schema_test_cases("1.0.2rc1.post2", "release/1"));

        // Step 10: Continue release branch development with dirty state and commits
        test_info!("Step 10: Continue release branch development with dirty state and commits");
        let scenario = scenario
            .make_dirty()
            .expect_version(
                "1.0.2-rc.1.post.1.dev.{timestamp:now}+release.1.0.g{hex:7}",
                "1.0.2rc1.post1.dev{timestamp:now}+release.1.0.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Rc,
                "1",
                1,
                Some("{timestamp:now}"),
                "release.1",
                0,
            ))
            .commit()
            .expect_version(
                "1.0.2-rc.1.post.1+release.1.1.g{hex:7}",
                "1.0.2rc1.post1+release.1.1.g{hex:7}",
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Rc,
                "1",
                1,
                None,
                "release.1",
                1,
            ))
            .create_tag("v1.0.2-rc.1.post.3")
            .expect_version("1.0.2-rc.1.post.3", "1.0.2rc1.post3");
        // .expect_schema_variants(create_base_schema_test_cases(
        //     "1.0.2rc1.post3",
        //     "release/1",
        // ));

        // Step 11: Final release merge to main
        test_info!("Step 11: Final release merge to main and release v1.1.0");
        let scenario = scenario
            .checkout("main")
            .merge_branch("release/1")
            .create_tag("v1.1.0")
            .debug_git_state("After creating v1.1.0 tag")
            .copy_test_path_to_cache("v1.1.0-tag-created")
            .expect_version("1.1.0", "1.1.0")
            .expect_schema_variants(create_base_schema_test_cases("1.1.0", "main"));

        // Step 12: Sync develop with release for next cycle
        test_info!("Step 12: Sync develop with release and prepare for next cycle");
        let scenario = scenario
            .checkout("develop")
            .merge_branch("main")
            .expect_version("1.1.0", "1.1.0")
            .expect_schema_variants(create_base_schema_test_cases("1.1.0", "develop"));

        test_info!("GitFlow test completed successfully - full scenario implemented");

        drop(scenario); // Test completes successfully
    }
}
