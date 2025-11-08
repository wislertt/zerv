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

        test_info!("Scenario 1: Initial setup - main branch state with v1.0.0 tag");
        let scenario = FlowTestScenario::new()
            .expect("Failed to create test scenario")
            .create_tag("v1.0.0")
            .expect_version("1.0.0", "1.0.0")
            .expect_schema_variants(create_base_schema_test_cases("1.0.0", "main"));

        test_info!("Scenario 2: Create feature branch and verify version expectations");
        let scenario = scenario
            .create_branch("feature-1")
            .checkout("feature-1")
            .expect_version("1.0.0", "1.0.0")
            .expect_schema_variants(create_base_schema_test_cases("1.0.0", "feature.1"));

        let branch_feature_1_hash = expect_branch_hash("feature-1", 5, "42954");

        test_info!("Scenario 3: Make working directory dirty and test all schema variants");
        let scenario = scenario
            .make_dirty()
            .expect_version(
                &format!(
                    "1.0.1-alpha.{}.post.0.dev.{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
                &format!(
                    "1.0.1a{}.post0.dev{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
            )
            .expect_schema_variants(create_full_schema_test_cases(
                "1.0.1",
                PreReleaseLabel::Alpha,
                &branch_feature_1_hash.to_string(),
                0,
                Some("{timestamp:now}"),
                "feature.1",
                0,
            ));

        test_info!(
            "Scenario 4: Make commit and test version expectations with post=1 and distance=1"
        );
        scenario
            .commit()
            .expect_version(
                &format!(
                    "1.0.1-alpha.{}.post.1+feature.1.1.{{hex:7}}",
                    branch_feature_1_hash
                ),
                &format!(
                    "1.0.1a{}.post1+feature.1.1.{{hex:7}}",
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
    }
}
