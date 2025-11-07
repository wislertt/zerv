use clap::Parser;

use crate::utils::constants::{
    formats,
    sources,
};

/// Reusable input configuration for version data
#[derive(Parser, Debug, Clone)]
pub struct InputConfig {
    // ============================================================================
    // INPUT OPTIONS
    // ============================================================================
    /// Input source for version data
    #[arg(short = 's', long = "source", default_value = sources::GIT, value_parser = [sources::GIT, sources::STDIN],
          help = "Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)")]
    pub source: String,

    /// Input format for version string parsing
    #[arg(short = 'f', long = "input-format", default_value = formats::AUTO, value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440],
          help = "Input format: 'auto' (detect), 'semver', or 'pep440'")]
    pub input_format: String,

    /// Working directory (default: current directory)
    #[arg(short = 'C', long = "directory", value_name = "DIR")]
    pub directory: Option<String>,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            source: sources::GIT.to_string(),
            input_format: formats::AUTO.to_string(),
            directory: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_config_defaults() {
        let config = InputConfig::default();
        assert_eq!(config.source, sources::GIT);
        assert_eq!(config.input_format, formats::AUTO);
        assert!(config.directory.is_none());
    }

    #[test]
    fn test_input_config_construction() {
        let config = InputConfig {
            source: sources::STDIN.to_string(),
            input_format: formats::SEMVER.to_string(),
            directory: Some("/path/to/repo".to_string()),
        };
        assert_eq!(config.source, sources::STDIN);
        assert_eq!(config.input_format, formats::SEMVER);
        assert_eq!(config.directory, Some("/path/to/repo".to_string()));
    }

    #[test]
    fn test_input_config_various_sources() {
        let sources_to_test = [
            (sources::GIT, sources::GIT),
            (sources::STDIN, sources::STDIN),
        ];

        for (source_value, expected_source) in sources_to_test {
            let config = InputConfig {
                source: source_value.to_string(),
                input_format: formats::AUTO.to_string(),
                directory: None,
            };
            assert_eq!(config.source, expected_source);
        }
    }

    #[test]
    fn test_input_config_various_formats() {
        let formats_to_test = [
            (formats::AUTO, formats::AUTO),
            (formats::SEMVER, formats::SEMVER),
            (formats::PEP440, formats::PEP440),
        ];

        for (format_value, expected_format) in formats_to_test {
            let config = InputConfig {
                source: sources::GIT.to_string(),
                input_format: format_value.to_string(),
                directory: None,
            };
            assert_eq!(config.input_format, expected_format);
        }
    }

    #[test]
    fn test_input_config_debug_format() {
        let config = InputConfig {
            source: "stdin".to_string(),
            input_format: "semver".to_string(),
            directory: Some("/test".to_string()),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("stdin"));
        assert!(debug_str.contains("semver"));
        assert!(debug_str.contains("/test"));
    }

    #[test]
    fn test_input_config_clone() {
        let config = InputConfig {
            source: "stdin".to_string(),
            input_format: "semver".to_string(),
            directory: Some("/test".to_string()),
        };
        let cloned = config.clone();
        assert_eq!(config.source, cloned.source);
        assert_eq!(config.input_format, cloned.input_format);
        assert_eq!(config.directory, cloned.directory);
    }

    #[test]
    fn test_input_config_empty_directory() {
        let config = InputConfig {
            source: sources::GIT.to_string(),
            input_format: formats::AUTO.to_string(),
            directory: Some("".to_string()),
        };
        assert_eq!(config.directory, Some("".to_string()));
    }

    #[test]
    fn test_input_config_complex_directory() {
        let complex_path = "/workspace/user/project/subdir";
        let config = InputConfig {
            source: sources::GIT.to_string(),
            input_format: formats::SEMVER.to_string(),
            directory: Some(complex_path.to_string()),
        };
        assert_eq!(config.directory, Some(complex_path.to_string()));
    }
}
