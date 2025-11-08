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
            bump_dev: args.bump_dev(),
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
        SchemaTestCase,
        create_all_standard_schema_test_cases,
        expect_branch_hash,
        test_flow_pipeline_with_fixture,
        test_flow_pipeline_with_schema_test_cases,
    };
    use crate::test_info;
    use crate::test_utils::{
        GitRepoFixture,
        should_run_docker_tests,
    };
    use crate::version::zerv::PreReleaseLabel;

    #[test]
    fn test_trunk_based_development_flow() {
        test_info!("Starting trunk-based development flow test");
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        let fixture =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create git fixture with tag");

        let fixture_path = fixture.path().to_string_lossy();

        test_flow_pipeline_with_fixture(&fixture_path, "1.0.0", "1.0.0");

        fixture
            .checkout_branch("feature-1")
            .expect("Failed to checkout feature-1 branch");

        test_flow_pipeline_with_fixture(&fixture_path, "1.0.0", "1.0.0");

        let branch_feature_1_hash = expect_branch_hash("feature-1", 5, "42954");

        fixture
            .make_dirty()
            .expect("Failed to make working directory dirty");

        test_flow_pipeline_with_fixture(
            &fixture_path,
            &format!(
                "1.0.0-alpha.{branch_feature_1_hash}.post.0.dev.{{timestamp:now}}+feature.1.0.{{hex:7}}"
            ),
            &format!(
                "1.0.0a{branch_feature_1_hash}.post0.dev{{timestamp:now}}+feature.1.0.{{hex:7}}"
            ),
        );

        let schema_test_cases = vec![
            SchemaTestCase {
                name: "standard-base",
                semver_expectation: "1.0.0".to_string(),
                pep440_expectation: "1.0.0".to_string(),
            },
            SchemaTestCase {
                name: "standard-base-prerelease",
                semver_expectation: format!("1.0.0-alpha.{}", branch_feature_1_hash),
                pep440_expectation: format!("1.0.0a{}", branch_feature_1_hash),
            },
            SchemaTestCase {
                name: "standard-base-prerelease-post",
                semver_expectation: format!("1.0.0-alpha.{}.post.0", branch_feature_1_hash),
                pep440_expectation: format!("1.0.0a{}.post0", branch_feature_1_hash),
            },
            SchemaTestCase {
                name: "standard-no-context",
                semver_expectation: format!(
                    "1.0.0-alpha.{}.post.0.dev.{{timestamp:now}}",
                    branch_feature_1_hash
                ),
                pep440_expectation: format!(
                    "1.0.0a{}.post0.dev{{timestamp:now}}",
                    branch_feature_1_hash
                ),
            },
            SchemaTestCase {
                name: "standard-context",
                semver_expectation: format!(
                    "1.0.0-alpha.{}.post.0.dev.{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
                pep440_expectation: format!(
                    "1.0.0a{}.post0.dev{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
            },
            SchemaTestCase {
                name: "standard",
                semver_expectation: format!(
                    "1.0.0-alpha.{}.post.0.dev.{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
                pep440_expectation: format!(
                    "1.0.0a{}.post0.dev{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
            },
        ];
        test_flow_pipeline_with_schema_test_cases(&fixture_path, schema_test_cases);
    }

    #[test]
    fn test_trunk_based_development_flow_builder() {
        test_info!("Starting trunk-based development flow test (builder pattern)");
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        // Initial setup - main branch state
        let scenario = FlowTestScenario::new()
            .expect("Failed to create test scenario")
            .expect_version("1.0.0", "1.0.0");

        // Create feature branch and verify version expectations
        let scenario = scenario
            .create_branch("feature-1")
            .expect_version("1.0.0", "1.0.0");

        // Make working directory dirty and test schema variants
        let branch_feature_1_hash = expect_branch_hash("feature-1", 5, "42954");

        scenario
            .make_dirty()
            .expect_version(
                &format!(
                    "1.0.0-alpha.{}.post.0.dev.{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
                &format!(
                    "1.0.0a{}.post0.dev{{timestamp:now}}+feature.1.0.{{hex:7}}",
                    branch_feature_1_hash
                ),
            )
            .expect_schema_variants(create_all_standard_schema_test_cases(
                "1.0.0",
                PreReleaseLabel::Alpha,
                &branch_feature_1_hash.to_string(),
                0,
                "feature.1",
                0,
            ));
    }
}
