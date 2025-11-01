use ron::from_str;

use crate::cli::common::args::OutputConfig;
use crate::cli::flow::args::FlowArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::version::args::{
    BumpsConfig,
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
        main: Default::default(),
        overrides: Default::default(),
        // bumps: Default::default(),
        bumps: BumpsConfig {
            bump_pre_release_label: args.bump_pre_release_label(),
            bump_pre_release_num: args.bump_pre_release_num(),
            bump_patch: args.bump_patch(),
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
    use crate::cli::utils::template::Template;
    use crate::test_info;
    use crate::test_utils::{
        GitRepoFixture,
        assert_version_expectation,
        should_run_docker_tests,
    };

    fn test_flow_pipeline_with_fixture(
        fixture_path: &str,
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

            let result = run_flow_pipeline(args);
            assert!(
                result.is_ok(),
                "Flow pipeline should succeed with {} format at {}: {}",
                format_name,
                fixture_path,
                result.unwrap_err()
            );
            let output = result.unwrap();
            assert!(
                !output.is_empty(),
                "Flow pipeline should produce output for {} format",
                format_name
            );

            assert_version_expectation(expectation, &output);

            test_info!("Flow pipeline output ({}): {}", format_name, output);
        }
    }

    #[test]
    fn test_trunk_based_development_flow() {
        test_info!("Starting trunk-based development flow test");
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        let fixture =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create git fixture with tag");

        let fixture_path = fixture.path().to_string_lossy();
        let _main_hash = Template::render("{{hash_int 'main' 5}}");

        test_flow_pipeline_with_fixture(&fixture_path, "1.0.0", "1.0.0");

        fixture
            .checkout_branch("feature-1")
            .expect("Failed to checkout feature-1 branch");

        test_flow_pipeline_with_fixture(&fixture_path, "1.0.0", "1.0.0");

        let feature_1_hash = Template::render("{{hash_int 'feature-1' 5}}");

        fixture
            .make_dirty()
            .expect("Failed to make working directory dirty");

        test_flow_pipeline_with_fixture(
            &fixture_path,
            &format!(
                "1.0.0-alpha.{}+feature.1.0.{{{{commit_hash_7}}}}",
                feature_1_hash
            ),
            &format!("1.0.0a{}+feature.1.0.{{{{commit_hash_7}}}}", feature_1_hash),
        );
    }
}
