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
    use super::*;
    use crate::cli::utils::template::{
        Template,
        TemplateExtGeneric,
    };
    use crate::test_info;
    use crate::test_utils::{
        GitRepoFixture,
        should_run_docker_tests,
    };

    mod test_utils {
        use super::*;
        use crate::test_utils::assert_version_expectation;

        // Test case structure for better readability and type safety
        #[derive(Debug)]
        pub struct SchemaTestCase {
            pub name: &'static str,
            pub semver_expectation: String,
            pub pep440_expectation: String,
        }

        pub fn test_flow_pipeline_with_fixture(
            fixture_path: &str,
            semver_expectation: &str,
            pep440_expectation: &str,
        ) {
            test_flow_pipeline_with_fixture_and_schema_opt(
                fixture_path,
                None,
                semver_expectation,
                pep440_expectation,
            )
        }

        pub fn test_flow_pipeline_with_fixture_and_schema(
            fixture_path: &str,
            schema: &str,
            semver_expectation: &str,
            pep440_expectation: &str,
        ) {
            test_flow_pipeline_with_fixture_and_schema_opt(
                fixture_path,
                Some(schema),
                semver_expectation,
                pep440_expectation,
            )
        }

        pub fn test_flow_pipeline_with_fixture_and_schema_opt(
            fixture_path: &str,
            schema: Option<&str>,
            semver_expectation: &str,
            pep440_expectation: &str,
        ) {
            let test_cases = vec![
                ("semver", semver_expectation),
                ("pep440", pep440_expectation),
            ];

            for (format_name, expectation) in test_cases {
                let mut args = FlowArgs::default();
                args.input.directory = Some(fixture_path.to_string());
                args.output.output_format = format_name.to_string();

                // Set schema if provided
                if let Some(schema_value) = schema {
                    args.schema = Some(schema_value.to_string());
                }

                let result = run_flow_pipeline(args);
                let schema_desc = match schema {
                    Some(s) => format!(" and {} schema", s),
                    None => String::new(),
                };

                assert!(
                    result.is_ok(),
                    "Flow pipeline should succeed with {} format{} at {}: {}",
                    format_name,
                    schema_desc,
                    fixture_path,
                    result.unwrap_err()
                );

                let output = result.unwrap();
                assert!(
                    !output.is_empty(),
                    "Flow pipeline should produce output for {} format{}",
                    format_name,
                    schema_desc
                );

                assert_version_expectation(expectation, &output);

                let log_msg = match schema {
                    Some(s) => format!("{} with {} schema", format_name, s),
                    None => format_name.to_string(),
                };
                test_info!("Flow pipeline output ({}): {}", log_msg, output);
            }
        }

        pub fn test_flow_pipeline_with_schema_test_cases(
            fixture_path: &str,
            schema_test_cases: Vec<SchemaTestCase>,
        ) {
            for test_case in schema_test_cases {
                test_flow_pipeline_with_fixture_and_schema(
                    fixture_path,
                    test_case.name,
                    &test_case.semver_expectation,
                    &test_case.pep440_expectation,
                );
            }
        }
    }

    use test_utils::{
        SchemaTestCase,
        test_flow_pipeline_with_fixture,
        test_flow_pipeline_with_schema_test_cases,
    };

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

        let branch_feature_1_hash =
            Template::<u32>::new("{{ hash_int(value='feature-1', length=5) }}".to_string())
                .render_unwrap(None);
        assert_eq!(branch_feature_1_hash, "42954");

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
}
