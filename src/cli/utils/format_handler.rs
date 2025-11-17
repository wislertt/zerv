use crate::error::ZervError;
use crate::version::Zerv;

pub struct InputFormatHandler;

impl InputFormatHandler {
    /// Parse Zerv RON format from input string
    pub fn parse_and_validate_zerv_ron(input: &str) -> Result<Zerv, ZervError> {
        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            return Err(ZervError::StdinError(
                "Empty input provided. When using --source stdin, provide valid Zerv RON format."
                    .to_string(),
            ));
        }

        // Try to parse as RON with detailed error information
        ron::from_str::<Zerv>(trimmed_input).map_err(|e| {
            ZervError::StdinError(format!(
                "Invalid Zerv RON format: {e}. Expected format: (vars: {{...}}, schema: {{...}})"
            ))
        })
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

    #[test]
    fn test_parse_and_validate_zerv_ron_handles_empty_input() {
        // Test that empty string returns appropriate error
        let result = InputFormatHandler::parse_and_validate_zerv_ron("");
        assert!(result.is_err(), "Should reject empty input");

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::StdinError(_)));
        assert!(error.to_string().contains("Empty input provided"));

        // Test that whitespace-only string returns appropriate error
        let result = InputFormatHandler::parse_and_validate_zerv_ron("   \n\t  ");
        assert!(result.is_err(), "Should reject whitespace-only input");

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::StdinError(_)));
        assert!(error.to_string().contains("Empty input provided"));
    }

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
}
