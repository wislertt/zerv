use clap::Parser;

use crate::cli::utils::template::Template;
use crate::utils::constants::formats;

/// Reusable output configuration for version strings
#[derive(Parser, Debug, Clone)]
pub struct OutputConfig {
    // ============================================================================
    // OUTPUT OPTIONS
    // ============================================================================
    /// Output format for generated version
    #[arg(long, default_value = formats::SEMVER, value_parser = formats::SUPPORTED_FORMATS_ARRAY,
          help = format!("Output format: '{}' (default), '{}', or '{}' (RON format for piping)", formats::SEMVER, formats::PEP440, formats::ZERV))]
    pub output_format: String,

    /// Output template for custom formatting (Tera syntax: {{ variable }})
    #[arg(
        long,
        help = "Output template for custom formatting (Tera syntax: {{ variable }})"
    )]
    pub output_template: Option<Template<String>>,

    /// Prefix to add to output
    #[arg(
        long,
        help = "Prefix to add to version output (e.g., 'v' for 'v1.0.0')"
    )]
    pub output_prefix: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_config_defaults() {
        let config = OutputConfig::default();
        assert_eq!(config.output_format, formats::SEMVER);
        assert!(config.output_template.is_none());
        assert!(config.output_prefix.is_none());
    }

    #[test]
    fn test_output_config_construction() {
        let config = OutputConfig {
            output_format: formats::PEP440.to_string(),
            output_template: Some(Template::new("v{{major}}.{{minor}}".to_string())),
            output_prefix: Some("release-".to_string()),
        };
        assert_eq!(config.output_format, formats::PEP440);
        assert!(config.output_template.is_some());
        assert_eq!(config.output_prefix, Some("release-".to_string()));
    }

    #[test]
    fn test_output_config_various_formats() {
        let formats_to_test = [
            (formats::SEMVER, formats::SEMVER),
            (formats::PEP440, formats::PEP440),
            (formats::ZERV, formats::ZERV),
        ];

        for (format_value, expected_format) in formats_to_test {
            let config = OutputConfig {
                output_format: format_value.to_string(),
                output_template: None,
                output_prefix: None,
            };
            assert_eq!(config.output_format, expected_format);
        }
    }

    #[test]
    fn test_output_config_with_template_construction() {
        let template_str = "v{{major}}.{{minor}}";
        let config = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::new(template_str.to_string())),
            output_prefix: None,
        };
        assert!(config.output_template.is_some());
        if let Some(template) = &config.output_template {
            assert_eq!(template.content(), template_str);
        }
    }

    #[test]
    fn test_output_config_with_prefix_construction() {
        let config = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: Some("v".to_string()),
        };
        assert_eq!(config.output_prefix, Some("v".to_string()));
    }

    #[test]
    fn test_output_config_all_options_construction() {
        let template_str = "{{version}}-{{distance}}";
        let config = OutputConfig {
            output_format: formats::ZERV.to_string(),
            output_template: Some(Template::new(template_str.to_string())),
            output_prefix: Some("build-".to_string()),
        };
        assert_eq!(config.output_format, formats::ZERV);
        assert!(config.output_template.is_some());
        assert_eq!(config.output_prefix, Some("build-".to_string()));
    }

    #[test]
    fn test_output_config_debug_format() {
        let config = OutputConfig {
            output_format: "pep440".to_string(),
            output_template: Some(Template::new("v{{major}}".to_string())),
            output_prefix: Some("release-".to_string()),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("pep440"));
        assert!(debug_str.contains("v{{major}}"));
        assert!(debug_str.contains("release-"));
    }

    #[test]
    fn test_output_config_clone() {
        let config = OutputConfig {
            output_format: "zerv".to_string(),
            output_template: Some(Template::new("{{version}}".to_string())),
            output_prefix: Some("build-".to_string()),
        };
        let cloned = config.clone();
        assert_eq!(config.output_format, cloned.output_format);
        assert_eq!(config.output_template, cloned.output_template);
        assert_eq!(config.output_prefix, cloned.output_prefix);
    }

    #[test]
    fn test_output_config_empty_prefix() {
        let config = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: None,
            output_prefix: Some("".to_string()),
        };
        assert_eq!(config.output_prefix, Some("".to_string()));
    }

    #[test]
    fn test_output_config_template_content_construction() {
        let template_str = "v{{major}}.{{minor}}.{{patch}}";
        let config = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::new(template_str.to_string())),
            output_prefix: None,
        };

        if let Some(template) = &config.output_template {
            assert_eq!(template.content(), template_str);
        } else {
            panic!("Expected Template::new with the template string");
        }
    }

    #[test]
    fn test_output_config_complex_template_construction() {
        let complex_template = "v{{major}}.{{minor}}.{{patch}}-{{pre_release}}+{{build}}";
        let config = OutputConfig {
            output_format: formats::SEMVER.to_string(),
            output_template: Some(Template::new(complex_template.to_string())),
            output_prefix: None,
        };

        if let Some(template) = &config.output_template {
            assert_eq!(template.content(), complex_template);
        } else {
            panic!("Expected Template::new with complex template string");
        }
    }
}
