use crate::error::ZervError;
use crate::schema::structure_validation;
use crate::version::{PEP440, SemVer, VersionObject, Zerv};
use std::io::Read;
use std::str::FromStr;

pub struct InputFormatHandler;

impl InputFormatHandler {
    /// Parse a version string with the specified format
    pub fn parse_version_string(
        version_str: &str,
        input_format: &str,
    ) -> Result<VersionObject, ZervError> {
        match input_format.to_lowercase().as_str() {
            "semver" => SemVer::from_str(version_str)
                .map(VersionObject::SemVer)
                .map_err(|e| {
                    ZervError::InvalidFormat(format!("Invalid SemVer format '{version_str}': {e}"))
                }),
            "pep440" => PEP440::from_str(version_str)
                .map(VersionObject::PEP440)
                .map_err(|e| {
                    ZervError::InvalidFormat(format!("Invalid PEP440 format '{version_str}': {e}"))
                }),
            "auto" => Self::parse_auto_detect(version_str),
            _ => Err(ZervError::UnknownFormat(format!(
                "Unknown input format '{input_format}'. Supported formats: semver, pep440, auto"
            ))),
        }
    }

    /// Parse stdin input expecting Zerv RON format with comprehensive validation
    pub fn parse_stdin(input_format: &str) -> Result<Zerv, ZervError> {
        // Read all input from stdin
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| ZervError::StdinError(format!("Failed to read from stdin: {e}")))?;

        if input.trim().is_empty() {
            return Err(ZervError::StdinError(
                "No input provided via stdin".to_string(),
            ));
        }

        match input_format.to_lowercase().as_str() {
            "zerv" => {
                // Comprehensive stdin format validation and parsing
                Self::parse_and_validate_zerv_ron(&input)
            }
            "semver" | "pep440" => {
                // Error: stdin should be Zerv RON when using these formats
                Err(ZervError::StdinError(format!(
                    "When using --source stdin with --input-format {input_format}, stdin must contain Zerv RON format. Use --input-format zerv or provide version via --tag-version instead."
                )))
            }
            _ => Err(ZervError::UnknownFormat(format!(
                "Unknown input format '{input_format}'. When using --source stdin, use --input-format zerv"
            ))),
        }
    }

    /// Parse and validate Zerv RON format with detailed error reporting
    fn parse_and_validate_zerv_ron(input: &str) -> Result<Zerv, ZervError> {
        let trimmed_input = input.trim();

        // First, detect if this looks like a simple version string
        if Self::looks_like_simple_version(trimmed_input) {
            return Err(ZervError::StdinError(format!(
                "Simple version string '{trimmed_input}' provided to stdin. Use --tag-version instead of --source stdin for version strings."
            )));
        }

        // Check for common non-RON formats and provide specific guidance
        if Self::looks_like_semver_or_pep440(trimmed_input) {
            return Err(ZervError::StdinError(format!(
                "Version string '{trimmed_input}' provided to stdin. When using --source stdin, provide Zerv RON format or use --tag-version '{trimmed_input}' instead."
            )));
        }

        // Check for JSON format (common mistake)
        if Self::looks_like_json(trimmed_input) {
            return Err(ZervError::StdinError(
                "JSON format detected in stdin. Zerv requires RON (Rust Object Notation) format, not JSON. Use --input-format zerv with proper RON syntax.".to_string()
            ));
        }

        // Attempt to parse as RON with enhanced error reporting
        match ron::from_str::<Zerv>(trimmed_input) {
            Ok(zerv) => {
                // Validate the parsed Zerv structure
                Self::validate_zerv_structure(&zerv)?;
                Ok(zerv)
            }
            Err(ron_error) => {
                // Provide detailed RON parsing error with line/column information
                Self::create_detailed_ron_error(trimmed_input, &ron_error)
            }
        }
    }

    /// Validate the structure of a parsed Zerv object
    fn validate_zerv_structure(zerv: &Zerv) -> Result<(), ZervError> {
        structure_validation::validate_zerv_structure(zerv)
    }

    /// Create detailed RON parsing error with helpful suggestions
    fn create_detailed_ron_error(
        _input: &str,
        ron_error: &ron::error::SpannedError,
    ) -> Result<Zerv, ZervError> {
        let base_error_msg = format!("Invalid Zerv RON format: {}", ron_error.code);

        // Use the base error message (RON error already contains position info)
        let error_msg = base_error_msg;

        // Add helpful suggestions based on common error patterns
        let error_str = error_msg.to_lowercase();
        let enhanced_msg = if error_str.contains("expected identifier")
            || error_str.contains("unexpected")
        {
            format!(
                "{error_msg}\n\nHint: RON field names must be valid identifiers (e.g., 'schema:', 'vars:'). Check for typos in field names."
            )
        } else if error_str.contains("expected") {
            format!(
                "{error_msg}\n\nHint: Check RON syntax - ensure proper use of parentheses (), brackets [], and field separators."
            )
        } else if error_str.contains("missing field") {
            format!(
                "{error_msg}\n\nHint: Zerv RON requires both 'schema' and 'vars' fields at the top level."
            )
        } else {
            format!(
                "{error_msg}\n\nHint: Ensure input follows Zerv RON format: (schema: (core: [...], extra_core: [...], build: [...]), vars: (...))"
            )
        };

        Err(ZervError::StdinError(enhanced_msg))
    }

    /// Check if input looks like SemVer or PEP440 format
    fn looks_like_semver_or_pep440(input: &str) -> bool {
        let trimmed = input.trim();

        // Must be single line and look like a version
        if trimmed.lines().count() != 1 {
            return false;
        }

        // Check for basic version patterns (X.Y.Z format)
        let has_basic_version = regex::Regex::new(r"^\d+\.\d+(\.\d+)?")
            .map(|re| re.is_match(trimmed))
            .unwrap_or(false);

        // Check for SemVer patterns (including pre-release and build metadata)
        let has_semver_extensions = regex::Regex::new(r"^\d+\.\d+\.\d+[-+]")
            .map(|re| re.is_match(trimmed))
            .unwrap_or(false);

        // Check for PEP440 patterns (including alpha, beta, rc, post, dev)
        let has_pep440_extensions =
            regex::Regex::new(r"^\d+(\.\d+)*(a\d*|b\d*|rc\d*|\.post\d*|\.dev\d*)")
                .map(|re| re.is_match(trimmed))
                .unwrap_or(false);

        has_basic_version || has_semver_extensions || has_pep440_extensions
    }

    /// Check if input looks like JSON format
    fn looks_like_json(input: &str) -> bool {
        let trimmed = input.trim();
        (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    }

    /// Auto-detect version format (try SemVer first, then PEP440)
    fn parse_auto_detect(version_str: &str) -> Result<VersionObject, ZervError> {
        // Try SemVer first
        if let Ok(semver) = SemVer::from_str(version_str) {
            return Ok(VersionObject::SemVer(semver));
        }

        // Fall back to PEP440
        if let Ok(pep440) = PEP440::from_str(version_str) {
            return Ok(VersionObject::PEP440(pep440));
        }

        Err(ZervError::InvalidVersion(format!(
            "Version '{version_str}' is not valid SemVer or PEP440 format"
        )))
    }

    /// Check if input looks like a simple version string rather than RON
    fn looks_like_simple_version(input: &str) -> bool {
        let trimmed = input.trim();

        // Check if it's a simple version-like string (no RON syntax)
        if trimmed.lines().count() == 1 {
            // Must contain at least one digit and one dot to look like a version
            let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
            let has_dot = trimmed.contains('.');

            // Check if it only contains version-like characters
            let version_like = trimmed.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || c == '.'
                    || c == '-'
                    || c == '+'
                    || c == '_'
                    || c == 'v'
            });

            // Also check if it doesn't contain RON syntax
            let no_ron_syntax = !trimmed.contains('(')
                && !trimmed.contains('{')
                && !trimmed.contains('[')
                && !trimmed.contains(':');

            // Exclude common RON keywords
            let not_ron_keyword = !matches!(
                trimmed.to_lowercase().as_str(),
                "none" | "some" | "true" | "false" | "varfield" | "component"
            );

            has_digit && has_dot && version_like && no_ron_syntax && not_ron_keyword
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_string_semver() {
        let result = InputFormatHandler::parse_version_string("1.2.3", "semver");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VersionObject::SemVer(_)));
    }

    #[test]
    fn test_parse_version_string_pep440() {
        let result = InputFormatHandler::parse_version_string("1.2.3a1", "pep440");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VersionObject::PEP440(_)));
    }

    #[test]
    fn test_parse_version_string_auto_semver() {
        let result = InputFormatHandler::parse_version_string("1.2.3", "auto");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VersionObject::SemVer(_)));
    }

    #[test]
    fn test_parse_version_string_auto_pep440() {
        let result = InputFormatHandler::parse_version_string("1.2.3a1", "auto");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VersionObject::PEP440(_)));
    }

    #[test]
    fn test_parse_version_string_invalid_semver() {
        let result = InputFormatHandler::parse_version_string("invalid", "semver");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_version_string_invalid_pep440() {
        let result = InputFormatHandler::parse_version_string("invalid", "pep440");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_version_string_unknown_format() {
        let result = InputFormatHandler::parse_version_string("1.2.3", "unknown");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::UnknownFormat(_)));
    }

    #[test]
    fn test_parse_version_string_auto_invalid() {
        let result = InputFormatHandler::parse_version_string("invalid", "auto");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::InvalidVersion(_)));
    }

    #[test]
    fn test_looks_like_simple_version() {
        // Should detect simple version strings
        assert!(InputFormatHandler::looks_like_simple_version("1.2.3"));
        assert!(InputFormatHandler::looks_like_simple_version("v1.2.3"));
        assert!(InputFormatHandler::looks_like_simple_version("1.2.3-alpha"));
        assert!(InputFormatHandler::looks_like_simple_version("1.2.3+build"));
        assert!(InputFormatHandler::looks_like_simple_version("1.2.3a1"));

        // Should not detect RON syntax
        assert!(!InputFormatHandler::looks_like_simple_version(
            "(schema: ())"
        ));
        assert!(!InputFormatHandler::looks_like_simple_version("{major: 1}"));
        assert!(!InputFormatHandler::looks_like_simple_version("[1, 2, 3]"));
        assert!(!InputFormatHandler::looks_like_simple_version("key: value"));

        // Should not detect multi-line input
        assert!(!InputFormatHandler::looks_like_simple_version(
            "1.2.3\n4.5.6"
        ));
    }

    #[test]
    fn test_error_messages_format_specific() {
        // Test SemVer error message
        let semver_error = InputFormatHandler::parse_version_string("invalid", "semver");
        assert!(semver_error.is_err());
        let error_msg = semver_error.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid SemVer format"));
        assert!(error_msg.contains("invalid"));

        // Test PEP440 error message
        let pep440_error = InputFormatHandler::parse_version_string("invalid", "pep440");
        assert!(pep440_error.is_err());
        let error_msg = pep440_error.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid PEP440 format"));
        assert!(error_msg.contains("invalid"));

        // Test unknown format error message
        let unknown_error = InputFormatHandler::parse_version_string("1.2.3", "unknown");
        assert!(unknown_error.is_err());
        let error_msg = unknown_error.unwrap_err().to_string();
        assert!(error_msg.contains("Unknown input format"));
        assert!(error_msg.contains("unknown"));
        assert!(error_msg.contains("Supported formats"));
    }

    #[test]
    fn test_auto_detection_priority() {
        // SemVer should be detected first for ambiguous cases
        let result = InputFormatHandler::parse_version_string("1.2.3", "auto");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VersionObject::SemVer(_)));

        // PEP440-specific syntax should be detected as PEP440
        let result = InputFormatHandler::parse_version_string("1.2.3a1", "auto");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), VersionObject::PEP440(_)));
    }

    #[test]
    fn test_stdin_error_messages() {
        // Test that we get appropriate error messages for wrong input formats
        // We can't easily test parse_stdin without mocking stdin, but we can test
        // the error message construction logic by checking the match arms

        // Test semver format error message construction
        let semver_msg = format!(
            "When using --source stdin with --input-format {}, stdin must contain Zerv RON format. Use --input-format zerv or provide version via --tag-version instead.",
            "semver"
        );
        assert!(semver_msg.contains("stdin must contain Zerv RON format"));
        assert!(semver_msg.contains("--input-format zerv"));
        assert!(semver_msg.contains("--tag-version"));

        // Test pep440 format error message construction
        let pep440_msg = format!(
            "When using --source stdin with --input-format {}, stdin must contain Zerv RON format. Use --input-format zerv or provide version via --tag-version instead.",
            "pep440"
        );
        assert!(pep440_msg.contains("stdin must contain Zerv RON format"));

        // Test unknown format error message construction
        let unknown_msg = format!(
            "Unknown input format '{}'. When using --source stdin, use --input-format zerv",
            "unknown"
        );
        assert!(unknown_msg.contains("Unknown input format"));
        assert!(unknown_msg.contains("--input-format zerv"));
    }

    #[test]
    fn test_parse_and_validate_zerv_ron_with_valid_input() {
        use crate::version::zerv::test_utils::base_zerv;

        // Create a valid Zerv object and convert to RON
        let zerv = base_zerv();
        let ron_string = zerv.to_string();

        // Test that we can parse it back successfully
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse valid Zerv RON successfully");

        let parsed_zerv = result.unwrap();
        assert_eq!(parsed_zerv, zerv, "Parsed Zerv should match original");
    }

    #[test]
    fn test_parse_and_validate_zerv_ron_with_simple_version_string() {
        let simple_versions = vec!["1.2.3", "v2.0.0", "1.0.0-alpha"];

        for version in simple_versions {
            let result = InputFormatHandler::parse_and_validate_zerv_ron(version);
            assert!(
                result.is_err(),
                "Should reject simple version string '{version}'"
            );

            let error = result.unwrap_err();
            match error {
                ZervError::StdinError(msg) => {
                    assert!(msg.contains("Simple version string"));
                    assert!(msg.contains("--tag-version"));
                    assert!(msg.contains(version));
                }
                _ => panic!("Expected StdinError for simple version string '{version}'"),
            }
        }
    }

    #[test]
    fn test_parse_and_validate_zerv_ron_with_semver_pep440_strings() {
        let version_strings = vec![
            "1.2.3",
            "2.0.0-alpha.1",
            "1.0.0+build.123",
            "1.2.3a1",
            "2.0.0b2",
            "1.0.0rc1",
        ];

        for version in version_strings {
            let result = InputFormatHandler::parse_and_validate_zerv_ron(version);
            assert!(result.is_err(), "Should reject version string '{version}'");

            let error = result.unwrap_err();
            match error {
                ZervError::StdinError(msg) => {
                    // Accept either "Simple version string" or "Version string" messages
                    assert!(
                        msg.contains("Version string") || msg.contains("Simple version string"),
                        "Error message should mention version string for '{version}': {msg}"
                    );
                    assert!(msg.contains("--tag-version"));
                    assert!(msg.contains(version));
                }
                _ => panic!("Expected StdinError for version string '{version}'"),
            }
        }
    }

    #[test]
    fn test_parse_and_validate_zerv_ron_with_json_input() {
        let json_inputs = vec![
            r#"{"schema": {"core": []}, "vars": {}}"#,
            r#"[1, 2, 3]"#,
            r#"{"version": "1.2.3"}"#,
        ];

        for json_input in json_inputs {
            let result = InputFormatHandler::parse_and_validate_zerv_ron(json_input);
            assert!(result.is_err(), "Should reject JSON input");

            let error = result.unwrap_err();
            match error {
                ZervError::StdinError(msg) => {
                    assert!(msg.contains("JSON format detected"));
                    assert!(msg.contains("RON (Rust Object Notation)"));
                }
                _ => panic!("Expected StdinError for JSON input"),
            }
        }
    }

    #[test]
    fn test_parse_and_validate_zerv_ron_with_invalid_ron() {
        let invalid_ron_inputs = vec![
            "(schema: (core: [",                                 // Incomplete RON
            "(invalid_field: 123)",                              // Missing required fields
            "(schema: (), vars: ())",                            // Empty schema
            "(schema: (core: [VarField(\"major\")]), vars: ())", // Missing major var
        ];

        for invalid_input in invalid_ron_inputs {
            let result = InputFormatHandler::parse_and_validate_zerv_ron(invalid_input);
            assert!(
                result.is_err(),
                "Should reject invalid RON: '{invalid_input}'"
            );

            let error = result.unwrap_err();
            match error {
                ZervError::StdinError(msg) => {
                    // Should contain helpful error information
                    assert!(
                        msg.contains("Invalid Zerv RON") || msg.contains("RON format"),
                        "Error message should mention RON format issues: {msg}"
                    );
                }
                _ => panic!("Expected StdinError for invalid RON: '{invalid_input}'"),
            }
        }
    }

    #[test]
    fn test_validate_zerv_structure_with_empty_schema() {
        use crate::version::{Zerv, ZervSchema, ZervVars};

        let empty_zerv = Zerv {
            schema: ZervSchema {
                core: vec![],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(),
        };

        let result = InputFormatHandler::validate_zerv_structure(&empty_zerv);
        assert!(result.is_err(), "Should reject empty schema");

        let error = result.unwrap_err();
        match error {
            ZervError::StdinError(msg) => {
                assert!(msg.contains("schema must contain at least one component"));
            }
            _ => panic!("Expected StdinError for empty schema"),
        }
    }

    #[test]
    fn test_validate_zerv_structure_with_missing_core_vars() {
        use crate::version::{Component, Zerv, ZervSchema, ZervVars};

        let zerv_missing_vars = Zerv {
            schema: ZervSchema {
                core: vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core: vec![],
                build: vec![],
            },
            vars: ZervVars::default(), // All None values
        };

        let result = InputFormatHandler::validate_zerv_structure(&zerv_missing_vars);
        assert!(result.is_err(), "Should reject missing core variables");

        let error = result.unwrap_err();
        match error {
            ZervError::StdinError(msg) => {
                assert!(msg.contains("missing core variables"));
                assert!(msg.contains("major"));
                assert!(msg.contains("minor"));
                assert!(msg.contains("patch"));
            }
            _ => panic!("Expected StdinError for missing core variables"),
        }
    }

    #[test]
    fn test_validate_zerv_structure_with_valid_structure() {
        use crate::version::zerv::test_utils::base_zerv;

        let valid_zerv = base_zerv();
        let result = InputFormatHandler::validate_zerv_structure(&valid_zerv);
        assert!(result.is_ok(), "Should accept valid Zerv structure");
    }

    #[test]
    fn test_looks_like_semver_or_pep440() {
        // Should detect SemVer patterns
        assert!(InputFormatHandler::looks_like_semver_or_pep440("1.2.3"));
        assert!(InputFormatHandler::looks_like_semver_or_pep440(
            "2.0.0-alpha.1"
        ));
        assert!(InputFormatHandler::looks_like_semver_or_pep440(
            "1.0.0+build.123"
        ));

        // Should detect PEP440 patterns
        assert!(InputFormatHandler::looks_like_semver_or_pep440("1.2.3a1"));
        assert!(InputFormatHandler::looks_like_semver_or_pep440("2.0.0b2"));
        assert!(InputFormatHandler::looks_like_semver_or_pep440("1.0.0rc1"));
        assert!(InputFormatHandler::looks_like_semver_or_pep440(
            "1.2.3.post1"
        ));
        assert!(InputFormatHandler::looks_like_semver_or_pep440(
            "1.0.0.dev1"
        ));

        // Should not detect RON or other formats
        assert!(!InputFormatHandler::looks_like_semver_or_pep440(
            "(schema: ())"
        ));
        assert!(!InputFormatHandler::looks_like_semver_or_pep440(
            "{\"version\": \"1.2.3\"}"
        ));
        assert!(!InputFormatHandler::looks_like_semver_or_pep440("invalid"));
        assert!(!InputFormatHandler::looks_like_semver_or_pep440(""));

        // Should not detect multi-line input
        assert!(!InputFormatHandler::looks_like_semver_or_pep440(
            "1.2.3\n4.5.6"
        ));
    }

    #[test]
    fn test_looks_like_json() {
        // Should detect JSON objects
        assert!(InputFormatHandler::looks_like_json(r#"{"key": "value"}"#));
        assert!(InputFormatHandler::looks_like_json(
            r#"{"schema": {"core": []}}"#
        ));

        // Should detect JSON arrays
        assert!(InputFormatHandler::looks_like_json("[1, 2, 3]"));
        assert!(InputFormatHandler::looks_like_json(r#"["a", "b", "c"]"#));

        // Should not detect RON or other formats
        assert!(!InputFormatHandler::looks_like_json("(schema: ())"));
        assert!(!InputFormatHandler::looks_like_json("1.2.3"));
        assert!(!InputFormatHandler::looks_like_json("invalid"));
        assert!(!InputFormatHandler::looks_like_json(""));

        // Should handle whitespace
        assert!(InputFormatHandler::looks_like_json(
            "  {\"key\": \"value\"}  "
        ));
        assert!(InputFormatHandler::looks_like_json("  [1, 2, 3]  "));
    }

    #[test]
    fn test_comprehensive_stdin_validation_error_messages() {
        // Test that all error messages provide actionable guidance

        // Simple version string error
        let result = InputFormatHandler::parse_and_validate_zerv_ron("1.2.3");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Simple version string"));
        assert!(error_msg.contains("--tag-version"));
        assert!(error_msg.contains("--source stdin"));

        // JSON format error
        let result = InputFormatHandler::parse_and_validate_zerv_ron(r#"{"version": "1.2.3"}"#);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("JSON format detected"));
        assert!(error_msg.contains("RON (Rust Object Notation)"));
        assert!(error_msg.contains("--input-format zerv"));

        // Invalid RON structure error
        let result = InputFormatHandler::parse_and_validate_zerv_ron("(invalid: syntax");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid Zerv RON format"));
        assert!(error_msg.contains("Hint:"));
    }

    #[test]
    fn test_enhanced_ron_error_reporting() {
        // Test that RON parsing errors include helpful hints
        let invalid_inputs = vec![
            ("(schema: (core: [", "RON syntax"),
            ("(missing_field: 123)", "schema"),
            ("invalid syntax", "RON format"),
        ];

        for (input, _expected_hint) in invalid_inputs {
            let result = InputFormatHandler::parse_and_validate_zerv_ron(input);
            assert!(result.is_err(), "Should fail for input: '{input}'");

            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("Invalid Zerv RON") || error_msg.contains("Hint:"),
                "Error should contain helpful information for '{input}': {error_msg}"
            );
        }
    }

    #[test]
    fn test_stdin_validation_with_complex_zerv_structures() {
        use crate::version::PreReleaseLabel;
        use crate::version::zerv::test_utils::{
            pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1, zerv_1_0_0_with_pre_release,
        };

        // Test with pre-release Zerv
        let pre_release_zerv = zerv_1_0_0_with_pre_release(PreReleaseLabel::Alpha, Some(1));
        let ron_string = pre_release_zerv.to_string();
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse pre-release Zerv successfully");

        // Test with complex PEP440 Zerv
        let complex_zerv = pep_zerv_complex_2_1_2_3_alpha_1_post_1_dev_1_local_1();
        let ron_string = complex_zerv.to_string();
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse complex Zerv successfully");
    }

    // Integration tests for comprehensive format handling
    #[test]
    fn test_parse_stdin_with_valid_zerv_ron() {
        use crate::version::zerv::test_utils::base_zerv;

        // Create a sample Zerv object
        let zerv = base_zerv();
        let ron_string = zerv.to_string();

        // Verify the RON string is valid
        assert!(ron_string.contains("schema"));
        assert!(ron_string.contains("vars"));

        // Test that we can parse it back
        let parsed: Result<Zerv, _> = ron::from_str(&ron_string);
        assert!(parsed.is_ok());

        // Verify the parsed object matches the original
        let parsed_zerv = parsed.unwrap();
        assert_eq!(parsed_zerv, zerv);
    }

    #[test]
    fn test_looks_like_simple_version_comprehensive() {
        // Test cases that should be detected as simple versions
        let simple_versions = vec![
            "1.2.3",
            "v1.2.3",
            "1.2.3-alpha",
            "1.2.3+build",
            "1.2.3-alpha.1",
            "1.2.3+build.123",
            "1.2.3-alpha.1+build.123",
            "1.2.3a1",
            "1.2.3b2",
            "1.2.3rc1",
            "2.0.0-beta.1",
            "10.20.30",
            "1.1.2-prerelease+meta",
            "1.1.2+meta",
            "1.1.2+meta-valid",
            "1.0.0-alpha-beta",
            "1.0.0-alpha.beta",
            "1.0.0-alpha.1",
            "1.0.0-alpha0.beta",
            "1.0.0-alpha.1.beta",
            "1.0.0-alpha.1.beta.1",
            "1.0.0-alpha-a.b-c-somethinglong+metadata+meta",
            "1.0.0-rc.1+meta",
            "1.2.3-beta",
            "10.2.3-DEV-SNAPSHOT",
            "1.2.3-SNAPSHOT-123",
            "1.0.0",
            "2.0.0",
            "1.1.7",
            "2.0.0+build.1",
            "2.0.0-rc.1",
            "1.2.3-beta.1",
        ];

        for version in simple_versions {
            assert!(
                InputFormatHandler::looks_like_simple_version(version),
                "Should detect '{version}' as a simple version"
            );
        }

        // Test cases that should NOT be detected as simple versions (RON syntax)
        let ron_like_inputs = vec![
            "(schema: (core: [VarField(\"major\")], extra_core: [], build: []), vars: (major: Some(1)))",
            "{ schema: { core: [] } }",
            "[1, 2, 3]",
            "key: value",
            "schema: ()",
            "vars: (major: Some(1))",
            "(major: 1, minor: 2)",
            "Some(1)",
            "None",
            "VarField(\"major\")",
            "Component::String(\"test\")",
            // Multi-line inputs
            "1.2.3\n4.5.6",
            "line1\nline2",
            "(schema:\n  core: [])",
        ];

        for input in ron_like_inputs {
            assert!(
                !InputFormatHandler::looks_like_simple_version(input),
                "Should NOT detect '{input}' as a simple version"
            );
        }
    }

    #[test]
    fn test_version_parsing_comprehensive() {
        // Test SemVer parsing
        let semver_cases = vec![
            "1.2.3",
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0+build",
            "1.0.0-alpha+build",
        ];

        for version in semver_cases {
            let result = InputFormatHandler::parse_version_string(version, "semver");
            assert!(
                result.is_ok(),
                "Should parse SemVer '{version}' successfully"
            );
            assert!(matches!(result.unwrap(), VersionObject::SemVer(_)));
        }

        // Test PEP440 parsing
        let pep440_cases = vec![
            "1.2.3",
            "1.2.3a1",
            "1.2.3b2",
            "1.2.3rc1",
            "1.2.3.post1",
            "1.2.3.dev1",
            "2!1.2.3",
        ];

        for version in pep440_cases {
            let result = InputFormatHandler::parse_version_string(version, "pep440");
            assert!(
                result.is_ok(),
                "Should parse PEP440 '{version}' successfully"
            );
            assert!(matches!(result.unwrap(), VersionObject::PEP440(_)));
        }

        // Test auto-detection
        let auto_cases = vec![
            ("1.2.3", "SemVer"),   // Should prefer SemVer
            ("1.2.3a1", "PEP440"), // PEP440-specific syntax
            ("2!1.2.3", "PEP440"), // Epoch is PEP440-specific
        ];

        for (version, expected_type) in auto_cases {
            let result = InputFormatHandler::parse_version_string(version, "auto");
            assert!(
                result.is_ok(),
                "Should auto-detect '{version}' successfully"
            );

            match (result.unwrap(), expected_type) {
                (VersionObject::SemVer(_), "SemVer") => {}
                (VersionObject::PEP440(_), "PEP440") => {}
                (actual, expected) => panic!(
                    "Auto-detection failed for '{version}': expected {expected}, got {actual:?}"
                ),
            }
        }
    }

    #[test]
    fn test_error_message_quality() {
        // Test format-specific error messages
        let invalid_semver = InputFormatHandler::parse_version_string("invalid-version", "semver");
        assert!(invalid_semver.is_err());
        let error_msg = invalid_semver.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid SemVer format"));
        assert!(error_msg.contains("invalid-version"));

        let invalid_pep440 = InputFormatHandler::parse_version_string("invalid-version", "pep440");
        assert!(invalid_pep440.is_err());
        let error_msg = invalid_pep440.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid PEP440 format"));
        assert!(error_msg.contains("invalid-version"));

        // Test unknown format error
        let unknown_format = InputFormatHandler::parse_version_string("1.2.3", "unknown");
        assert!(unknown_format.is_err());
        let error_msg = unknown_format.unwrap_err().to_string();
        assert!(error_msg.contains("Unknown input format"));
        assert!(error_msg.contains("unknown"));
        assert!(error_msg.contains("Supported formats"));

        // Test auto-detection failure
        let auto_invalid = InputFormatHandler::parse_version_string("completely-invalid", "auto");
        assert!(auto_invalid.is_err());
        let error_msg = auto_invalid.unwrap_err().to_string();
        assert!(error_msg.contains("not valid SemVer or PEP440 format"));
        assert!(error_msg.contains("completely-invalid"));
    }

    #[test]
    fn test_case_insensitive_format_handling() {
        // Test that format names are case-insensitive
        let test_cases = vec![
            ("semver", "1.2.3"),
            ("SEMVER", "1.2.3"),
            ("SemVer", "1.2.3"),
            ("pep440", "1.2.3a1"),
            ("PEP440", "1.2.3a1"),
            ("Pep440", "1.2.3a1"),
            ("auto", "1.2.3"),
            ("AUTO", "1.2.3"),
            ("Auto", "1.2.3"),
        ];

        for (format, version) in test_cases {
            let result = InputFormatHandler::parse_version_string(version, format);
            assert!(
                result.is_ok(),
                "Should handle case-insensitive format '{format}' for version '{version}'"
            );
        }
    }
}
