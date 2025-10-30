use ron::from_str;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
};
use crate::cli::flow::args::FlowArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::version::args::VersionArgs;
use crate::cli::version::pipeline::run_version_pipeline;
use crate::error::ZervError;
use crate::utils::constants::*;

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
        input: InputConfig {
            source: sources::GIT.to_string(),
            input_format: "auto".to_string(),
            directory: None,
        },
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

    #[test]
    fn test_run_flow_pipeline_basic() {
        let args = FlowArgs::default();
        let result = run_flow_pipeline(args);
        // Should work now (assuming git repo exists)
        // Note: This test may fail in environments without git
        match result {
            Ok(_) => {} // Success
            Err(ZervError::VcsNotFound(_)) => {
                // Expected in test environments without git
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_run_flow_pipeline_different_output_formats() {
        let formats = ["semver", "pep440", "zerv"];

        for format in formats.iter() {
            let mut args = FlowArgs::default();
            args.output.output_format = format.to_string();

            let result = run_flow_pipeline(args);
            match result {
                Ok(_) => {} // Success
                Err(ZervError::VcsNotFound(_)) => {
                    // Expected in test environments without git
                }
                Err(e) => {
                    panic!("Unexpected error for format '{}': {:?}", format, e);
                }
            }
        }
    }

    #[test]
    fn test_run_flow_pipeline_with_output_prefix() {
        let mut args = FlowArgs::default();
        args.output.output_prefix = Some("v".to_string());

        let result = run_flow_pipeline(args);
        match result {
            Ok(output) => {
                // Output should start with the prefix
                assert!(
                    output.starts_with('v'),
                    "Output should start with prefix 'v': {}",
                    output
                );
            }
            Err(ZervError::VcsNotFound(_)) => {
                // Expected in test environments without git
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }
}
