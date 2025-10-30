use ron::from_str;

use crate::cli::common::args::OutputConfig;
use crate::cli::flow::args::FlowArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::version::args::VersionArgs;
use crate::cli::version::pipeline::run_version_pipeline;
use crate::error::ZervError;

/// Main flow pipeline handler
///
/// This function orchestrates the flow version generation process:
/// 1. Validate input arguments
/// 2. Call version command pipeline with zerv output format
/// 3. Parse RON output to Zerv object
/// 4. Format and return results
pub fn run_flow_pipeline(args: FlowArgs) -> Result<String, ZervError> {
    tracing::debug!("Starting flow pipeline with args: {:?}", args);

    // Validate arguments first
    let mut args = args;
    args.validate()?;

    // For now, just call version command with zerv output format
    // TODO: Phase 2+ - Translate flow arguments to version arguments
    let version_args = VersionArgs {
        input: args.input.clone(),
        output: OutputConfig {
            output_format: "zerv".to_string(),
            output_template: None,
            output_prefix: None,
        },
        main: Default::default(),
        overrides: Default::default(),
        bumps: Default::default(),
    };

    // Call version pipeline to get RON output
    let ron_output = run_version_pipeline(version_args)?;

    // Parse RON output to Zerv object
    let zerv_object: crate::version::zerv::core::Zerv = from_str(&ron_output)
        .map_err(|e| ZervError::InvalidFormat(format!("Failed to parse version output: {}", e)))?;

    // Format output using the same formatter as version command
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
                "Flow pipeline should succeed with {} format at {}",
                format_name,
                fixture_path
            );
            let output = result.unwrap();
            assert!(
                !output.is_empty(),
                "Flow pipeline should produce output for {} format",
                format_name
            );

            assert_version_expectation(expectation, &output);

            println!("✓ Flow pipeline output ({}): {}", format_name, output);
        }
    }

    #[test]
    fn test_trunk_based_development_flow() {
        if !should_run_docker_tests() {
            return; // Skip when `ZERV_TEST_DOCKER` are disabled
        }

        // Step 1: Start with a clean tag
        let fixture =
            GitRepoFixture::tagged("v1.0.0").expect("Failed to create git fixture with tag");
        let fixture_path = fixture.path().to_string_lossy();

        test_flow_pipeline_with_fixture(&fixture_path, r"1.0.0", r"1.0.0");

        // Step 2: Checkout feature branch
        fixture
            .checkout_branch("feature-1")
            .expect("Failed to checkout feature-1 branch");

        test_flow_pipeline_with_fixture(&fixture_path, r"1.0.0", r"1.0.0");

        // Step 3: Make dirty working directory
        fixture.make_dirty().expect("Failed to make fixture dirty");

        test_flow_pipeline_with_fixture(
            &fixture_path,
            r"1.0.0+feature.1.0.{{commit_hash_7}}",
            r"1.0.0+feature.1.0.{{commit_hash_7}}",
        );
    }
}
