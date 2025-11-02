use regex::{
    Regex,
    escape,
};

use crate::error::ZervError;

/// Parse a readable pattern with placeholders into a compiled regex
///
/// Supports:
/// - {commit_hash_7} -> [a-f0-9]{7}
/// - {regex:pattern} -> pattern (direct regex insertion)
/// - {{ and }} for literal braces
/// - All other parts are auto-escaped for exact matching
fn parse_readable_pattern(pattern: &str) -> Result<Regex, ZervError> {
    // Process pattern to handle double braces and placeholders
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            // Double brace for literal { - find the matching }}
            result.push('{');
            i += 2;

            // Find the matching }} for the content
            let mut j = i;
            while j < chars.len() {
                if j + 1 < chars.len() && chars[j] == '}' && chars[j + 1] == '}' {
                    // Found matching }}, add literal content and escape it
                    let literal_content: String = chars[i..j].iter().collect();
                    result.push_str(&escape(&literal_content));
                    result.push('}');
                    i = j + 2; // Skip past the }}
                    break;
                } else {
                    j += 1;
                }
            }

            if j >= chars.len() {
                return Err(ZervError::InvalidVersion(
                    "Unclosed double brace".to_string(),
                ));
            }
        } else if chars[i] == '{' {
            // Single brace - find the matching closing brace
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }

            if j >= chars.len() {
                return Err(ZervError::InvalidVersion(
                    "Unclosed placeholder brace".to_string(),
                ));
            }

            // Extract placeholder content
            let placeholder: String = chars[i + 1..j].iter().collect();

            // Process placeholder
            let regex_pattern = if placeholder.starts_with("hex") {
                // Hex parametrization: {hex:7} -> [a-f0-9]{7}, {hex} -> [a-f0-9]+
                if placeholder == "hex" {
                    "[a-f0-9]+".to_string()
                } else if let Some(length_str) = placeholder.strip_prefix("hex:") {
                    match length_str.parse::<usize>() {
                        Ok(length) => {
                            if length > 0 {
                                format!("[a-f0-9]{{{}}}", length)
                            } else {
                                return Err(ZervError::InvalidVersion(format!(
                                    "Hex length must be positive, got 0 in placeholder {{{}}}",
                                    placeholder
                                )));
                            }
                        }
                        Err(_) => {
                            return Err(ZervError::InvalidVersion(format!(
                                "Invalid hex length in placeholder {{{}}}. Expected format: {{hex}} or {{hex:number}}",
                                placeholder
                            )));
                        }
                    }
                } else {
                    return Err(ZervError::InvalidVersion(format!(
                        "Invalid hex placeholder format: {{{}}}. Expected: {{hex}} or {{hex:number}}",
                        placeholder
                    )));
                }
            } else if let Some(regex_pattern) = placeholder.strip_prefix("regex:") {
                regex_pattern.to_string() // Remove "regex:" prefix
            } else if placeholder.is_empty() {
                "".to_string() // Handle {} placeholders as empty string
            } else {
                return Err(ZervError::InvalidVersion(format!(
                    "Unknown placeholder: {{{}}}. Supported placeholders: {{hex}}, {{hex:number}}, {{regex:pattern}}, {{}} for empty, and {{{{}}}} for literal braces",
                    placeholder
                )));
            };

            result.push_str(&regex_pattern);
            i = j + 1;
        } else {
            // Regular character - escape it for literal matching
            result.push_str(&escape(&chars[i].to_string()));
            i += 1;
        }
    }

    // Add anchors for full match
    result = format!("^{}$", result);

    Regex::new(&result)
        .map_err(|e| ZervError::InvalidVersion(format!("Invalid regex pattern: {}", e)))
}

pub fn assert_version_expectation(expectation: &str, actual: &str) {
    let regex = parse_readable_pattern(expectation)
        .unwrap_or_else(|e| panic!("Failed to parse pattern '{}': {}", expectation, e));

    assert!(
        regex.is_match(actual),
        "Version assertion failed\nExpected pattern: '{}'\nActual:   '{}'\nCompiled regex: '{}'",
        expectation,
        actual,
        regex.as_str()
    );
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // Basic hex parametrization tests
    #[case("0.7.74+dev.4.{hex:7}", "0.7.74+dev.4.d4738bb")]
    #[case("1.0.0+dev.1.{hex:7}", "1.0.0+dev.1.a1b2c3d")]
    #[case("prefix-{hex:7}-suffix", "prefix-d4738bb-suffix")]
    #[case("{hex:7}-middle-{hex:7}", "d4738bb-middle-a1b2c3d")]
    #[case("{hex:7}{hex:7}{hex:7}", "d4738bba1b2c3dabc1234")]
    #[case("exact-match-no-placeholders", "exact-match-no-placeholders")]
    // Variable length hex tests
    #[case("{hex:8}", "a1b2c3d4")]
    #[case("{hex:12}", "deadbeefcafe")]
    #[case("{hex:40}", "0123456789abcdef0123456789abcdef01234567")]
    #[case("{hex}", "abcdef123456789")] // variable length
    #[case("build-{hex:8}-release", "build-a1b2c3d4-release")]
    // New functionality tests - custom regex placeholders
    #[case(
        "1.0.0-{regex:[a-z]+\\d+}+build.{hex:7}",
        "1.0.0-alpha123+build.a1b2c3d"
    )]
    #[case("build-{regex:\\d+}-release", "build-123-release")]
    // Special characters (auto-escaped)
    #[case("version-1.0.0+test.{hex:7}", "version-1.0.0+test.a1b2c3d")]
    #[case("build[1.0.0]{hex:7}", "build[1.0.0]abc1234")]
    #[case("file+name=1.0.0.{hex:7}", "file+name=1.0.0.d4738bb")]
    fn test_assert_version_expectation_function(#[case] expectation: &str, #[case] actual: &str) {
        assert_version_expectation(expectation, actual);
    }

    #[rstest]
    // Updated error tests - the error messages will be different now
    #[case(
        "0.7.74+dev.4.{hex:7}",
        "1.7.74+dev.4.d4738bb",
        "Version assertion failed"
    )]
    #[case(
        "0.7.74+dev.4.{hex:7}",
        "0.7.74+dev.4.xyz1234",
        "Version assertion failed"
    )]
    #[case("prefix-{hex:7}", "prefix-d4738bb-extra", "Version assertion failed")]
    fn test_assert_version_expectation_fail_cases(
        #[case] expectation: &str,
        #[case] actual: &str,
        #[case] expected_error: &str,
    ) {
        let result = std::panic::catch_unwind(|| {
            assert_version_expectation(expectation, actual);
        });
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        let error_str = error_msg.downcast_ref::<String>().unwrap();
        assert!(error_str.contains(expected_error));
    }

    #[test]
    fn test_parse_readable_pattern_basic() {
        let result = parse_readable_pattern("1.0.0-{hex:7}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("1.0.0-a1b2c3d"));
        assert!(!regex.is_match("1.0.0-xyz1234")); // not hex
    }

    #[test]
    fn test_parse_readable_pattern_custom_regex() {
        let result = parse_readable_pattern("1.0.0-{regex:[a-z]+\\d+}+build");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("1.0.0-alpha123+build"));
        assert!(!regex.is_match("1.0.0-ALPHA123+build")); // uppercase not allowed
        assert!(!regex.is_match("1.0.0-alpha+build")); // missing number
    }

    #[test]
    fn test_parse_readable_pattern_special_characters() {
        let result = parse_readable_pattern("version-1.0.0{regex:plus}test");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("version-1.0.0plustest"));
        assert!(!regex.is_match("version-1.0.0plustestx")); // extra character
    }

    #[test]
    fn test_parse_readable_pattern_no_placeholders() {
        let result = parse_readable_pattern("exact-match-1.0.0+build");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("exact-match-1.0.0+build"));
        assert!(!regex.is_match("exact-match-1.0.0+Build")); // case sensitive
        assert!(!regex.is_match("exact-match-1.0.0+build-123")); // extra characters
    }

    #[test]
    fn test_hex_parametrization() {
        // Test fixed length hex
        let result = parse_readable_pattern("{hex:7}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("a1b2c3d"));
        assert!(!regex.is_match("a1b2c")); // too short
        assert!(!regex.is_match("a1b2c3d8")); // too long
        assert!(!regex.is_match("xyz1234")); // not hex

        // Test variable length hex
        let result = parse_readable_pattern("{hex}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("a1b2c3d"));
        assert!(regex.is_match("deadbeefcafe1234"));
        assert!(!regex.is_match("xyz123")); // not hex

        // Test different lengths
        let result = parse_readable_pattern("{hex:12}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("deadbeefcafe"));
        assert!(!regex.is_match("deadbeef")); // too short
        assert!(!regex.is_match("deadbeefcafe1")); // too long
    }

    #[test]
    fn test_hex_parametrization_errors() {
        // Test invalid hex length (non-numeric)
        let result = parse_readable_pattern("{hex:abc}");
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = match error {
            ZervError::InvalidVersion(msg) => msg,
            _ => panic!("Expected InvalidVersion error"),
        };
        assert!(error_msg.contains("Invalid hex length"));

        // Test zero hex length
        let result = parse_readable_pattern("{hex:0}");
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = match error {
            ZervError::InvalidVersion(msg) => msg,
            _ => panic!("Expected InvalidVersion error"),
        };
        assert!(error_msg.contains("Hex length must be positive"));

        // Test invalid hex format
        let result = parse_readable_pattern("{hex_7}");
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = match error {
            ZervError::InvalidVersion(msg) => msg,
            _ => panic!("Expected InvalidVersion error"),
        };
        assert!(error_msg.contains("Invalid hex placeholder format"));
    }

    #[test]
    fn test_parse_readable_pattern_unknown_placeholder() {
        let result = parse_readable_pattern("1.0.0-{unknown_placeholder}");
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = match error {
            ZervError::InvalidVersion(msg) => msg,
            _ => panic!("Expected InvalidVersion error"),
        };
        assert!(error_msg.contains("Unknown placeholder"));
        assert!(error_msg.contains("{unknown_placeholder}"));
    }

    #[test]
    fn test_parse_readable_pattern_invalid_regex() {
        let result = parse_readable_pattern("1.0.0-{regex:[invalid}");
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = match error {
            ZervError::InvalidVersion(msg) => msg,
            _ => panic!("Expected InvalidVersion error"),
        };
        assert!(error_msg.contains("Invalid regex pattern"));
    }

    #[test]
    fn test_parse_readable_pattern_empty_placeholders() {
        let result = parse_readable_pattern("1.0.0-{}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("1.0.0-"));
    }
}
