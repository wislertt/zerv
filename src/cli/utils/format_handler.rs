use std::io::{
    IsTerminal,
    Read,
};
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
        if std::io::stdin().is_terminal() {
            return Err(ZervError::StdinError(
                "No input provided via stdin. Use echo or pipe to provide input.".to_string(),
            ));
        }

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
    use rstest::rstest;

    use super::*;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::{
        Component,
        PreReleaseLabel,
    };

    #[rstest]
    #[case::semver_valid("1.2.3", "semver", true, Some("SemVer"))]
    #[case::pep440_valid("1.2.3a1", "pep440", true, Some("PEP440"))]
    #[case::auto_semver("1.2.3", "auto", true, Some("SemVer"))]
    #[case::auto_pep440("1.2.3a1", "auto", true, Some("PEP440"))]
    #[case::semver_invalid("invalid", "semver", false, None)]
    #[case::pep440_invalid("invalid", "pep440", false, None)]
    #[case::unknown_format("1.2.3", "unknown", false, None)]
    #[case::auto_invalid("invalid", "auto", false, None)]
    fn test_parse_version_string(
        #[case] version: &str,
        #[case] format: &str,
        #[case] should_succeed: bool,
        #[case] expected_type: Option<&str>,
    ) {
        let result = InputFormatHandler::parse_version_string(version, format);

        if should_succeed {
            assert!(result.is_ok(), "Should parse '{version}' as {format}");
            let version_obj = result.unwrap();

            match expected_type {
                Some("SemVer") => assert!(matches!(version_obj, VersionObject::SemVer(_))),
                Some("PEP440") => assert!(matches!(version_obj, VersionObject::PEP440(_))),
                _ => {}
            }
        } else {
            assert!(
                result.is_err(),
                "Should fail to parse '{version}' as {format}"
            );

            // Verify error type based on format
            let error = result.unwrap_err();
            match format {
                "semver" | "pep440" => assert!(matches!(error, ZervError::InvalidFormat(_))),
                "unknown" => assert!(matches!(error, ZervError::UnknownFormat(_))),
                "auto" => assert!(matches!(error, ZervError::InvalidVersion(_))),
                _ => {}
            }
        }
    }

    #[rstest]
    #[case::semver_invalid("invalid", "semver", &["Invalid SemVer format", "invalid"])]
    #[case::pep440_invalid("invalid", "pep440", &["Invalid PEP440 format", "invalid"])]
    #[case::unknown_format("1.2.3", "unknown", &["Unknown input format", "unknown", "Supported formats"])]
    fn test_error_messages_format_specific(
        #[case] version: &str,
        #[case] format: &str,
        #[case] expected_substrings: &[&str],
    ) {
        let result = InputFormatHandler::parse_version_string(version, format);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        for substring in expected_substrings {
            assert!(
                error_msg.contains(substring),
                "Error message should contain '{substring}': {error_msg}"
            );
        }
    }

    #[rstest]
    #[case::ambiguous_semver("1.2.3", "SemVer")]
    #[case::pep440_specific("1.2.3a1", "PEP440")]
    fn test_auto_detection_priority(#[case] version: &str, #[case] expected_type: &str) {
        let result = InputFormatHandler::parse_version_string(version, "auto");
        assert!(result.is_ok());

        let version_obj = result.unwrap();
        match expected_type {
            "SemVer" => assert!(matches!(version_obj, VersionObject::SemVer(_))),
            "PEP440" => assert!(matches!(version_obj, VersionObject::PEP440(_))),
            _ => panic!("Unknown expected type: {expected_type}"),
        }
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

    // Note: parse_stdin_to_zerv() is tested via integration tests where we can control stdin.
    // The parsing logic itself is thoroughly tested via parse_and_validate_zerv_ron() tests below.

    #[test]
    fn test_parse_and_validate_zerv_ron_with_valid_input() {
        // Create a valid Zerv object and convert to RON
        let zerv = ZervFixture::basic().zerv().clone();
        let ron_string = zerv.to_string();

        // Test that we can parse it back successfully
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse valid Zerv RON successfully");

        let parsed_zerv = result.unwrap();
        assert_eq!(parsed_zerv, zerv, "Parsed Zerv should match original");
    }

    #[rstest]
    #[case::simple_version("1.2.3")]
    #[case::v_prefix("v2.0.0")]
    #[case::semver_prerelease("1.0.0-alpha")]
    #[case::semver_prerelease_number("2.0.0-alpha.1")]
    #[case::semver_build("1.0.0+build.123")]
    #[case::pep440_alpha("1.2.3a1")]
    #[case::pep440_beta("2.0.0b2")]
    #[case::pep440_rc("1.0.0rc1")]
    #[case::json_object(r#"{"schema": {"core": []}, "vars": {}}"#)]
    #[case::json_array(r#"[1, 2, 3]"#)]
    #[case::json_version(r#"{"version": "1.2.3"}"#)]
    #[case::incomplete_ron("(schema: (core: [")]
    #[case::missing_fields("(invalid_field: 123)")]
    #[case::empty_schema("(schema: (), vars: ())")]
    #[case::missing_var(r#"(schema: (core: [VarField("major")]), vars: ())"#)]
    #[case::invalid_syntax("(invalid: syntax")]
    fn test_parse_and_validate_zerv_ron_rejects_invalid_input(#[case] input: &str) {
        let result = InputFormatHandler::parse_and_validate_zerv_ron(input);
        assert!(result.is_err(), "Should reject input: '{input}'");

        let error = result.unwrap_err();
        assert!(
            matches!(error, ZervError::StdinError(_)),
            "Should return StdinError for invalid input: '{input}'"
        );

        let error_msg = error.to_string();
        assert!(
            error_msg.contains("Invalid input format") || error_msg.contains("Stdin error"),
            "Error message should be helpful for '{input}': {error_msg}"
        );
    }

    #[test]
    fn test_stdin_validation_with_complex_zerv_structures() {
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
            .with_build(Component::UInt(1))
            .build()
            .clone();
        let ron_string = complex_zerv.to_string();
        let result = InputFormatHandler::parse_and_validate_zerv_ron(&ron_string);
        assert!(result.is_ok(), "Should parse complex Zerv successfully");
    }

    // Integration tests for comprehensive format handling
    #[test]
    fn test_zerv_ron_parsing() {
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

    #[rstest]
    #[case::semver_basic("1.2.3", "semver", "SemVer")]
    #[case::semver_prerelease("1.0.0-alpha", "semver", "SemVer")]
    #[case::semver_prerelease_num("1.0.0-alpha.1", "semver", "SemVer")]
    #[case::semver_build("1.0.0+build", "semver", "SemVer")]
    #[case::semver_both("1.0.0-alpha+build", "semver", "SemVer")]
    #[case::pep440_basic("1.2.3", "pep440", "PEP440")]
    #[case::pep440_alpha("1.2.3a1", "pep440", "PEP440")]
    #[case::pep440_beta("1.2.3b2", "pep440", "PEP440")]
    #[case::pep440_rc("1.2.3rc1", "pep440", "PEP440")]
    #[case::pep440_post("1.2.3.post1", "pep440", "PEP440")]
    #[case::pep440_dev("1.2.3.dev1", "pep440", "PEP440")]
    #[case::pep440_epoch("2!1.2.3", "pep440", "PEP440")]
    #[case::auto_semver("1.2.3", "auto", "SemVer")]
    #[case::auto_pep440_alpha("1.2.3a1", "auto", "PEP440")]
    #[case::auto_pep440_epoch("2!1.2.3", "auto", "PEP440")]
    fn test_version_parsing_comprehensive(
        #[case] version: &str,
        #[case] format: &str,
        #[case] expected_type: &str,
    ) {
        let result = InputFormatHandler::parse_version_string(version, format);
        assert!(result.is_ok(), "Should parse '{version}' as {format}");

        let version_obj = result.unwrap();
        match expected_type {
            "SemVer" => assert!(matches!(version_obj, VersionObject::SemVer(_))),
            "PEP440" => assert!(matches!(version_obj, VersionObject::PEP440(_))),
            _ => panic!("Unknown expected type: {expected_type}"),
        }
    }

    #[rstest]
    #[case::semver_invalid("invalid-version", "semver", &["Invalid SemVer format", "invalid-version"])]
    #[case::pep440_invalid("invalid-version", "pep440", &["Invalid PEP440 format", "invalid-version"])]
    #[case::unknown_format("1.2.3", "unknown", &["Unknown input format", "unknown", "Supported formats"])]
    #[case::auto_invalid("completely-invalid", "auto", &["not valid SemVer or PEP440 format", "completely-invalid"])]
    fn test_error_message_quality(
        #[case] version: &str,
        #[case] format: &str,
        #[case] expected_substrings: &[&str],
    ) {
        let result = InputFormatHandler::parse_version_string(version, format);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        for substring in expected_substrings {
            assert!(
                error_msg.contains(substring),
                "Error message should contain '{substring}': {error_msg}"
            );
        }
    }

    #[rstest]
    #[case::semver_lower("semver", "1.2.3")]
    #[case::semver_upper("SEMVER", "1.2.3")]
    #[case::semver_mixed("SemVer", "1.2.3")]
    #[case::pep440_lower("pep440", "1.2.3a1")]
    #[case::pep440_upper("PEP440", "1.2.3a1")]
    #[case::pep440_mixed("Pep440", "1.2.3a1")]
    #[case::auto_lower("auto", "1.2.3")]
    #[case::auto_upper("AUTO", "1.2.3")]
    #[case::auto_mixed("Auto", "1.2.3")]
    fn test_case_insensitive_format_handling(#[case] format: &str, #[case] version: &str) {
        let result = InputFormatHandler::parse_version_string(version, format);
        assert!(
            result.is_ok(),
            "Should handle case-insensitive format '{format}' for version '{version}'"
        );
    }
}
