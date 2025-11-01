use clap::Parser;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
    Validation as CommonValidation,
};
use crate::error::ZervError;

/// Generate version with intelligent pre-release management based on Git branch patterns
#[derive(Parser)]
#[command(
    about = "Generate version with intelligent pre-release management based on Git branch patterns"
)]
#[command(
    long_about = "Generate version strings with automatic pre-release detection based on Git branch patterns.
This command acts as an intelligent wrapper around 'zerv version' that automatically determines
pre-release information from the current Git branch using alpha pre-release labels.

INPUT/OUTPUT OPTIONS:
  -s, --source <TYPE>       Input source: git, stdin
  -f, --input-format <FMT>  Input format: auto, semver, pep440
  -o, --output-format <FMT> Output format: semver, pep440, zerv
  -t, --output-template <TPL> Custom output template (Handlebars)
  -p, --output-prefix <PFX> Add prefix to version output

EXAMPLES:
  # Basic flow version with automatic pre-release detection
  zerv flow

  # Different output formats
  zerv flow --output-format pep440
  zerv flow --output-format zerv
  zerv flow --output-prefix v

  # Use in different directory
  zerv flow -C /path/to/repo"
)]
#[derive(Debug, Default)]
pub struct FlowArgs {
    #[command(flatten)]
    pub input: InputConfig,

    #[command(flatten)]
    pub output: OutputConfig,
}

impl FlowArgs {
    /// Validate arguments and return early errors
    /// This provides early validation before flow processing
    /// Note: source and format validation is handled by clap's value parser
    pub fn validate(&mut self) -> Result<(), ZervError> {
        // Use shared validation for input/output
        CommonValidation::validate_io(&self.input, &self.output)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::common::args::{
        InputConfig,
        OutputConfig,
    };

    #[test]
    fn test_flow_args_default() {
        let args = FlowArgs::default();
        assert_eq!(args.input.source, "git");
        assert_eq!(args.output.output_format, "semver");
    }

    #[test]
    fn test_flow_args_validation_success() {
        let mut args = FlowArgs::default();
        // Should validate successfully with defaults
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_flow_args_with_input_output() {
        let mut args = FlowArgs {
            input: InputConfig {
                source: "git".to_string(),
                input_format: "auto".to_string(),
                directory: Some("/test/path".to_string()),
            },
            output: OutputConfig {
                output_format: "zerv".to_string(),
                output_prefix: Some("v".to_string()),
                output_template: None,
            },
        };
        assert_eq!(args.input.source, "git");
        assert_eq!(args.output.output_format, "zerv");
        assert_eq!(args.output.output_prefix, Some("v".to_string()));
        assert!(args.validate().is_ok());
    }
}
