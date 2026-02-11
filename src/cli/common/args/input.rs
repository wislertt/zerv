use clap::Parser;
#[cfg(test)]
use rstest::rstest;

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
    #[arg(short = 's', long = "source", value_parser = [sources::GIT, sources::STDIN, sources::NONE],
          help = "Input source: 'git' (extract from repository), 'stdin' (read Zerv RON format), or 'none' (no source, use overrides only)")]
    pub source: Option<String>,

    /// Input format for version string parsing
    #[arg(short = 'f', long = "input-format", default_value = formats::AUTO, value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440],
          help = "Input format: 'auto' (detect), 'semver', or 'pep440'")]
    pub input_format: String,

    /// Working directory (default: current directory)
    #[arg(short = 'C', long = "directory", value_name = "DIR")]
    pub directory: Option<String>,
}

impl InputConfig {
    /// Apply smart source default: stdin if available, otherwise git
    /// This is called after stdin detection to determine the appropriate default source
    pub fn apply_smart_source_default(&mut self, has_stdin: bool) {
        if self.source.is_none() {
            self.source = if has_stdin {
                Some(sources::STDIN.to_string())
            } else {
                Some(sources::GIT.to_string())
            };
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            source: Some(sources::GIT.to_string()),
            input_format: formats::AUTO.to_string(),
            directory: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_config_construction() {
        let config = InputConfig {
            source: Some(sources::STDIN.to_string()),
            input_format: formats::SEMVER.to_string(),
            directory: Some("/path/to/repo".to_string()),
        };
        assert_eq!(config.source, Some(sources::STDIN.to_string()));
        assert_eq!(config.input_format, formats::SEMVER);
        assert_eq!(config.directory, Some("/path/to/repo".to_string()));
    }

    #[test]
    fn test_input_config_various_sources() {
        let sources_to_test = [
            (sources::GIT, sources::GIT),
            (sources::STDIN, sources::STDIN),
            (sources::NONE, sources::NONE),
        ];

        for (source_value, expected_source) in sources_to_test {
            let config = InputConfig {
                source: Some(source_value.to_string()),
                input_format: formats::AUTO.to_string(),
                directory: None,
            };
            assert_eq!(config.source.as_deref(), Some(expected_source));
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
                source: Some(sources::GIT.to_string()),
                input_format: format_value.to_string(),
                directory: None,
            };
            assert_eq!(config.input_format, expected_format);
        }
    }

    #[test]
    fn test_input_config_debug_format() {
        let config = InputConfig {
            source: Some("stdin".to_string()),
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
            source: Some("stdin".to_string()),
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
            source: Some(sources::GIT.to_string()),
            input_format: formats::AUTO.to_string(),
            directory: Some("".to_string()),
        };
        assert_eq!(config.directory, Some("".to_string()));
    }

    #[test]
    fn test_input_config_complex_directory() {
        let complex_path = "/workspace/user/project/subdir";
        let config = InputConfig {
            source: Some(sources::GIT.to_string()),
            input_format: formats::SEMVER.to_string(),
            directory: Some(complex_path.to_string()),
        };
        assert_eq!(config.directory, Some(complex_path.to_string()));
    }

    #[test]
    fn test_input_config_none_source() {
        let config = InputConfig {
            source: None,
            input_format: formats::AUTO.to_string(),
            directory: None,
        };
        assert!(config.source.is_none());
        assert_eq!(config.input_format, formats::AUTO);
        assert!(config.directory.is_none());
    }

    #[rstest]
    #[case::none_source_no_stdin(None, false, sources::GIT)]
    #[case::none_source_with_stdin(None, true, sources::STDIN)]
    #[case::explicit_git_no_stdin(Some(sources::GIT), false, sources::GIT)]
    #[case::explicit_git_with_stdin(Some(sources::GIT), true, sources::GIT)]
    #[case::explicit_stdin_no_stdin(Some(sources::STDIN), false, sources::STDIN)]
    #[case::explicit_stdin_with_stdin(Some(sources::STDIN), true, sources::STDIN)]
    #[case::explicit_none_no_stdin(Some(sources::NONE), false, sources::NONE)]
    #[case::explicit_none_with_stdin(Some(sources::NONE), true, sources::NONE)]
    fn test_apply_smart_source_default(
        #[case] initial_source: Option<&str>,
        #[case] has_stdin: bool,
        #[case] expected_source: &str,
    ) {
        let mut config = InputConfig {
            source: initial_source.map(|s| s.to_string()),
            input_format: formats::AUTO.to_string(),
            directory: None,
        };
        config.apply_smart_source_default(has_stdin);
        assert_eq!(config.source.as_deref(), Some(expected_source));
    }
}
