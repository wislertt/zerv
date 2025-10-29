use super::{
    InputConfig,
    OutputConfig,
};
use crate::error::ZervError;
use crate::utils::constants::formats;

/// Shared validation methods for input and output configurations
pub struct Validation;

impl Validation {
    /// Validate input configuration
    pub fn validate_input(_input: &InputConfig) -> Result<(), ZervError> {
        // Input validation is handled by clap's value parser for source and input_format
        // No additional validation needed for input config
        Ok(())
    }

    /// Validate output configuration
    pub fn validate_output(output: &OutputConfig) -> Result<(), ZervError> {
        // Output format validation is handled by clap's value parser

        // Check for conflicts between output template and output format
        if output.output_template.is_some() {
            if output.output_format != formats::SEMVER {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --output-template with --output-format. \
                     Use --output-format alone for pure format output, \
                     or --output-template alone for custom formatting"
                        .to_string(),
                ));
            }
            if output.output_prefix.is_some() {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --output-template with --output-prefix. \
                     Add the prefix directly in your template instead \
                     (e.g., 'v{{major}}.{{minor}}.{{patch}}')"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate both input and output configurations
    pub fn validate_io(input: &InputConfig, output: &OutputConfig) -> Result<(), ZervError> {
        Self::validate_input(input)?;
        Self::validate_output(output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::utils::template::Template;
    use crate::utils::constants::{
        formats,
        sources,
    };

    fn create_valid_input() -> InputConfig {
        InputConfig {
            source: sources::GIT.to_string(),
            input_format: formats::AUTO.to_string(),
            directory: Some("/test".to_string()),
        }
    }

    fn create_valid_output() -> OutputConfig {
        OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: None,
        }
    }

    #[test]
    fn test_validate_input_success() {
        let input = create_valid_input();
        assert!(Validation::validate_input(&input).is_ok());
    }

    #[test]
    fn test_validate_input_all_sources() {
        let sources_to_test = [sources::GIT, sources::STDIN];

        for source in sources_to_test {
            let input = InputConfig {
                source: source.to_string(),
                input_format: formats::AUTO.to_string(),
                directory: None,
            };
            assert!(Validation::validate_input(&input).is_ok());
        }
    }

    #[test]
    fn test_validate_input_all_formats() {
        let formats_to_test = [formats::AUTO, formats::SEMVER, formats::PEP440];

        for format in formats_to_test {
            let input = InputConfig {
                source: sources::GIT.to_string(),
                input_format: format.to_string(),
                directory: None,
            };
            assert!(Validation::validate_input(&input).is_ok());
        }
    }

    #[test]
    fn test_validate_output_success() {
        let output = create_valid_output();
        assert!(Validation::validate_output(&output).is_ok());
    }

    #[test]
    fn test_validate_output_all_formats() {
        let formats_to_test = [formats::SEMVER, formats::PEP440, formats::ZERV];

        for format in formats_to_test {
            let output = OutputConfig {
                output_format: format.to_string(),
                output_template: None,
                output_prefix: None,
            };
            assert!(Validation::validate_output(&output).is_ok());
        }
    }

    #[test]
    fn test_validate_output_with_prefix_success() {
        let output = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: Some("v".to_string()),
        };
        assert!(Validation::validate_output(&output).is_ok());
    }

    #[test]
    fn test_validate_output_template_with_semver_success() {
        let output = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::Value("v{{major}}.{{minor}}".to_string())),
            output_prefix: None,
        };
        assert!(Validation::validate_output(&output).is_ok());
    }

    #[test]
    fn test_validate_output_template_with_non_semver_fails() {
        let output = OutputConfig {
            output_format: formats::PEP440.to_string(),
            output_template: Some(Template::Value("v{{major}}.{{minor}}".to_string())),
            output_prefix: None,
        };
        let result = Validation::validate_output(&output);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ZervError::ConflictingOptions(_)
        ));
    }

    #[test]
    fn test_validate_output_template_with_prefix_fails() {
        let output = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::Value("v{{major}}.{{minor}}".to_string())),
            output_prefix: Some("release-".to_string()),
        };
        let result = Validation::validate_output(&output);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ZervError::ConflictingOptions(_)
        ));
    }

    #[test]
    fn test_validate_io_success() {
        let input = create_valid_input();
        let output = create_valid_output();
        assert!(Validation::validate_io(&input, &output).is_ok());
    }

    #[test]
    fn test_validate_io_propagates_output_error() {
        let input = create_valid_input();
        let output = OutputConfig {
            output_format: formats::PEP440.to_string(),
            output_template: Some(Template::Value("template".to_string())),
            output_prefix: None,
        };
        let result = Validation::validate_io(&input, &output);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ZervError::ConflictingOptions(_)
        ));
    }

    #[test]
    fn test_validate_output_error_message_template_format() {
        let output = OutputConfig {
            output_format: formats::PEP440.to_string(),
            output_template: Some(Template::Value("test".to_string())),
            output_prefix: None,
        };
        let result = Validation::validate_output(&output);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("output-template"));
        assert!(error_msg.contains("output-format"));
    }

    #[test]
    fn test_validate_output_error_message_template_prefix() {
        let output = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::Value("test".to_string())),
            output_prefix: Some("v".to_string()),
        };
        let result = Validation::validate_output(&output);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("output-template"));
        assert!(error_msg.contains("output-prefix"));
        assert!(error_msg.contains("v{{major}}.{{minor}}.{{patch}}"));
    }

    #[test]
    fn test_validate_input_with_directory() {
        let input = InputConfig {
            source: sources::GIT.to_string(),
            input_format: formats::AUTO.to_string(),
            directory: Some("/workspace/project".to_string()),
        };
        assert!(Validation::validate_input(&input).is_ok());
    }

    #[test]
    fn test_validate_output_zerv_format_with_template_fails() {
        let output = OutputConfig {
            output_format: formats::ZERV.to_string(),
            output_template: Some(Template::Value("template".to_string())),
            output_prefix: None,
        };
        let result = Validation::validate_output(&output);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ZervError::ConflictingOptions(_)
        ));
    }

    #[test]
    fn test_validate_output_edge_cases() {
        // Test with empty string prefix (should be valid)
        let output = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: Some("".to_string()),
        };
        assert!(Validation::validate_output(&output).is_ok());

        // Test with complex template (should be valid with semver)
        let output = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::Value(
                "v{{major}}.{{minor}}.{{patch}}-{{pre_release}}".to_string(),
            )),
            output_prefix: None,
        };
        assert!(Validation::validate_output(&output).is_ok());
    }
}
