use clap::Parser;

use crate::cli::common::args::{
    OutputConfig,
    Validation,
};
use crate::error::ZervError;
use crate::utils::constants::formats;

pub mod pipeline;

pub use pipeline::run_render;

/// Render a version string with format conversion and output options
#[derive(Parser, Debug, Clone)]
#[command(about = "Render a version string with flexible output options")]
#[command(
    long_about = "Parse a version string and render it with flexible output options.
Supports format conversion (SemVer ↔ PEP440), normalization, templates, and schemas."
)]
pub struct RenderArgs {
    /// Version string to render
    #[arg(required = true, value_name = "VERSION")]
    pub version: String,

    /// Input format (auto-detected if not specified)
    #[arg(
        short = 'f',
        long = "input-format",
        default_value = formats::AUTO,
        value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440, formats::ZERV],
        help = "Input format: 'auto' (default), 'semver', 'pep440', or 'zerv'"
    )]
    pub input_format: String,

    /// Output configuration (same as version/flow)
    #[command(flatten)]
    pub output: OutputConfig,
}

impl RenderArgs {
    pub fn validate(&self) -> Result<(), ZervError> {
        Validation::validate_output(&self.output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::cli::utils::template::Template;

    #[rstest]
    #[case("1.2.3", formats::AUTO)]
    #[case("2.0.0-alpha.1", formats::SEMVER)]
    #[case("1.2.3a1", formats::PEP440)]
    fn test_render_args_basic(#[case] version: &str, #[case] format: &str) {
        let args = RenderArgs {
            version: version.to_string(),
            input_format: format.to_string(),
            output: OutputConfig::default(),
        };
        assert_eq!(args.version, version);
        assert_eq!(args.input_format, format);
    }

    #[test]
    fn test_render_args_with_output_options() {
        let args = RenderArgs {
            version: "1.2.3".to_string(),
            input_format: formats::SEMVER.to_string(),
            output: OutputConfig {
                output_format: formats::SEMVER.to_string(),
                output_template: Some(Template::new("v{{major}}".to_string())),
                output_prefix: None,
            },
        };
        assert_eq!(args.version, "1.2.3");
        assert_eq!(args.input_format, formats::SEMVER);
        assert_eq!(args.output.output_format, formats::SEMVER);
        assert!(args.output.output_template.is_some());
        assert!(args.output.output_prefix.is_none());
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_render_args_with_prefix() {
        let args = RenderArgs {
            version: "1.2.3".to_string(),
            input_format: formats::SEMVER.to_string(),
            output: OutputConfig {
                output_format: formats::SEMVER.to_string(),
                output_template: None,
                output_prefix: Some("v".to_string()),
            },
        };
        assert_eq!(args.version, "1.2.3");
        assert_eq!(args.input_format, formats::SEMVER);
        assert_eq!(args.output.output_prefix, Some("v".to_string()));
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_render_args_template_with_prefix_fails() {
        let args = RenderArgs {
            version: "1.2.3".to_string(),
            input_format: formats::SEMVER.to_string(),
            output: OutputConfig {
                output_format: formats::SEMVER.to_string(),
                output_template: Some(Template::new("v{{major}}".to_string())),
                output_prefix: Some("release-".to_string()),
            },
        };
        assert!(args.validate().is_err());
        assert!(matches!(
            args.validate().unwrap_err(),
            ZervError::ConflictingOptions(_)
        ));
    }

    #[rstest]
    #[case(formats::AUTO, formats::AUTO)]
    #[case(formats::SEMVER, formats::SEMVER)]
    #[case(formats::PEP440, formats::PEP440)]
    #[case(formats::ZERV, formats::ZERV)]
    fn test_render_args_input_formats(#[case] format: &str, #[case] expected: &str) {
        let args = RenderArgs {
            version: "1.0.0".to_string(),
            input_format: format.to_string(),
            output: OutputConfig::default(),
        };
        assert_eq!(args.input_format, expected);
    }
}
