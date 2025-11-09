use ron::from_str;

use crate::cli::common::args::OutputConfig;
use crate::cli::flow::args::FlowArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::version::args::{
    BumpsConfig,
    MainConfig,
    VersionArgs,
};
use crate::cli::version::pipeline::run_version_pipeline;
use crate::error::ZervError;
use crate::version::zerv::core::Zerv;

pub fn run_flow_pipeline(args: FlowArgs) -> Result<String, ZervError> {
    tracing::debug!("Starting flow pipeline with args: {:?}", args);

    let mut args = args;
    args.validate()?;

    let bump_dev = args.bump_dev();

    let version_args = VersionArgs {
        input: args.input.clone(),
        output: OutputConfig {
            output_format: "zerv".to_string(),
            output_template: None,
            output_prefix: None,
        },
        main: MainConfig {
            schema: args.schema.clone(),
            schema_ron: None,
        },
        overrides: Default::default(),
        bumps: BumpsConfig {
            bump_pre_release_label: args.bump_pre_release_label(),
            bump_pre_release_num: args.bump_pre_release_num(),
            bump_patch: args.bump_patch(),
            bump_post: args.bump_post(),
            bump_dev,
            ..Default::default()
        },
    };

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
        test_info!("Starting trunk-based development flow test (builder pattern)");
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        test_info!("Initial setup: main branch state with v1.0.0 tag");
        let scenario = FlowTestScenario::new()
            .expect("Failed to create test scenario")
            .create_tag("v1.0.0")
            .expect_version("1.0.0", "1.0.0")
            .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

        test_info!("Create parallel feature branches: feature-1 and feature-2");
        let scenario = scenario
            .create_branch("feature-1")
            .create_branch("feature-2");

        // Capture actual hash values first, then validate against mermaid expectations
        let branch_feature_2_hash = expect_branch_hash("feature-2", 5, "68031"); // Update with actual
        let branch_feature_1_hash = expect_branch_hash("feature-1", 5, "42954"); // Update with actual

        test_info!("feature-2: Start development with dirty state");
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

        test_info!("feature-2: Create first commit");
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

        test_info!("feature-1: Create first commit");
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
            ));

        test_info!("feature-1: Create second commit");
        let scenario = scenario
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

        test_info!("feature-1: Merge to main and release v1.0.1");
        let scenario = scenario
            .checkout("main")
            .merge_branch("feature-1")
            .create_tag("v1.0.1")
            .expect_version("1.0.1", "1.0.1")
            .expect_schema_variants(create_base_schema_test_cases("1.0.1", "main"));

        test_info!("feature-2: Sync with main to get feature-1 changes");
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

        // DEBUG: Analyze Git state at this moment
        scenario.debug_git_state("after feature-2 merges main");

        test_info!("feature-3: Branch from feature-2 for sub-feature development");
        let branch_feature_3_hash = expect_branch_hash("feature-3", 5, "14698");
        let scenario = scenario
            .create_branch("feature-3")
            .checkout("feature-3")
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.2+feature.3.2.g{{hex:7}}",
                    branch_feature_3_hash
                ),
                &format!(
                    "1.0.2a{}.post2+feature.3.2.g{{hex:7}}",
                    branch_feature_3_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                2,
                None,
                "feature.3",
                2,
            ));

        test_info!("feature-3: Start development");
        let scenario = scenario
            .make_dirty()
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.2.dev.{{timestamp:now}}+feature.3.2.g{{hex:7}}",
                    branch_feature_3_hash
                ),
                &format!(
                    "1.0.2a{}.post2.dev{{timestamp:now}}+feature.3.2.g{{hex:7}}",
                    branch_feature_3_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                2,
                Some("{timestamp:now}"),
                "feature.3",
                2,
            ));

        test_info!("feature-3: Continue development");
        let scenario = scenario
            .commit()
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.3+feature.3.3.g{{hex:7}}",
                    branch_feature_3_hash
                ),
                &format!(
                    "1.0.2a{}.post3+feature.3.3.g{{hex:7}}",
                    branch_feature_3_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_3_hash.to_string(),
                3,
                None,
                "feature.3",
                3,
            ))
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

        test_info!("feature-2: Merge feature-3 back to continue development");
        let scenario = scenario
            .checkout("feature-2")
            .merge_branch("feature-3")
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.4+feature.2.4.g{{hex:7}}",
                    branch_feature_2_hash
                ),
                &format!(
                    "1.0.2a{}.post4+feature.2.4.g{{hex:7}}",
                    branch_feature_2_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                4,
                None,
                "feature.2",
                4,
            ));

        test_info!("feature-2: Final development before release");
        let scenario = scenario
            .commit()
            .expect_version(
                &format!(
                    "1.0.2-alpha.{}.post.5+feature.2.5.g{{hex:7}}",
                    branch_feature_2_hash
                ),
                &format!(
                    "1.0.2a{}.post5+feature.2.5.g{{hex:7}}",
                    branch_feature_2_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.2",
                PreReleaseLabel::Alpha,
                &branch_feature_2_hash.to_string(),
                5,
                None,
                "feature.2",
                5,
            ));

        test_info!("feature-2: Merge to main and release v1.1.0");
        scenario
            .checkout("main")
            .merge_branch("feature-2")
            .create_tag("v1.1.0")
            .expect_version("1.1.0", "1.1.0")
            .expect_schema_variants(create_base_schema_test_cases("1.1.0", "main"));
    }
}
