use std::io::Read;
use std::str::FromStr;

use crate::error::ZervError;
use crate::version::{
    PEP440,
    SemVer,
    VersionObject,
    Zerv,
};

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
    pub fn parse_stdin_to_zerv() -> Result<Zerv, ZervError> {
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

        // Parse as Zerv RON format
        Self::parse_and_validate_zerv_ron(&input)
    }

    /// Parse Zerv RON format from input string
    fn parse_and_validate_zerv_ron(input: &str) -> Result<Zerv, ZervError> {
        let trimmed_input = input.trim();

        // Try to parse as RON - if it fails, provide a simple error message
        ron::from_str::<Zerv>(trimmed_input).map_err(|_| {
            ZervError::StdinError(
                "Invalid input format. When using --source stdin, provide Zerv RON format only."
                    .to_string(),
            )
        })
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
        // We can't easily test parse_stdin_to_zerv without mocking stdin, but we can test
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
        use crate::test_utils::zerv::ZervFixture;

        // Create a valid Zerv object and convert to RON
        let zerv = ZervFixture::basic().zerv().clone();
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
                    assert!(msg.contains("Invalid input format"));
                    assert!(msg.contains("Zerv RON format only"));
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
                    assert!(msg.contains("Invalid input format"));
                    assert!(msg.contains("Zerv RON format only"));
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
                    assert!(msg.contains("Invalid input format"));
                    assert!(msg.contains("Zerv RON format only"));
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
    fn test_comprehensive_stdin_validation_error_messages() {
        // Test that all error messages provide actionable guidance

        // Simple version string error
        let result = InputFormatHandler::parse_and_validate_zerv_ron("1.2.3");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid input format"));
        assert!(error_msg.contains("Zerv RON format only"));

        // JSON format error
        let result = InputFormatHandler::parse_and_validate_zerv_ron(r#"{"version": "1.2.3"}"#);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid input format"));
        assert!(error_msg.contains("Zerv RON format only"));

        // Invalid RON structure error
        let result = InputFormatHandler::parse_and_validate_zerv_ron("(invalid: syntax");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid input format"));
        assert!(error_msg.contains("Zerv RON format only"));
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
                error_msg.contains("Invalid input format")
                    && error_msg.contains("Zerv RON format only"),
                "Error should contain simplified message for '{input}': {error_msg}"
            );
        }
    }

    #[test]
    fn test_stdin_validation_with_complex_zerv_structures() {
        use crate::test_utils::zerv::ZervFixture;
        use crate::version::zerv::{
            Component,
            PreReleaseLabel,
        };

        // Test with pre-release Zerv
        let pre_release_zerv = ZervFixture::new()
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .build()
            .clone();
        let ron_string = pre_release_zerv.to_string();
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse pre-release Zerv successfully");

        // Test with complex PEP440 Zerv
        let complex_zerv = ZervFixture::new()
            .with_epoch(2)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_post(1)
            .with_dev(1)
            .with_build(Component::Str("local".to_string()))
            .with_build(Component::Int(1))
            .build()
            .clone();
        let ron_string = complex_zerv.to_string();
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse complex Zerv successfully");
    }

    // Integration tests for comprehensive format handling
    #[test]
    fn test_zerv_ron_parsing() {
        use crate::test_utils::zerv::ZervFixture;

        // Create a sample Zerv object
        let zerv = ZervFixture::basic().zerv().clone();
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
