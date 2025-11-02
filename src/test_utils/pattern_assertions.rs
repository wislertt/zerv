use regex::{
    Regex,
    escape,
};

use crate::error::ZervError;

/// Parse a readable pattern with placeholders into a compiled regex
///
/// Supports:
/// - {hex:length} -> [a-f0-9]{length}
/// - {hex} -> [a-f0-9]+
/// - {regex:pattern} -> pattern (direct regex insertion)
/// - Non-placeholder characters are escaped only if they're regex-special
/// - For literal braces: use {regex:\{pattern\}}
fn parse_readable_pattern(pattern: &str) -> Result<Regex, ZervError> {
    let result = parse_pattern_tokens(pattern)?;
    let anchored = format!("^{}$", result);
    Regex::new(&anchored)
        .map_err(|e| ZervError::InvalidVersion(format!("Invalid regex pattern: {}", e)))
}

/// Parse pattern by processing tokens and building regex string
fn parse_pattern_tokens(pattern: &str) -> Result<String, ZervError> {
    let mut result = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match process_next_token(&chars, &mut i) {
            Ok(token) => result.push_str(&token),
            Err(e) => return Err(e),
        }
    }

    Ok(result)
}

/// Process the next token in the pattern starting at position i
/// Returns the processed token string and updates i to point to the next position
fn process_next_token(chars: &[char], i: &mut usize) -> Result<String, ZervError> {
    if chars[*i] == '{' {
        process_single_brace(chars, i)
    } else {
        // Regular character - escape only regex-special characters
        let char_str = chars[*i].to_string();
        let result = if is_regex_special(&chars[*i]) {
            escape(&char_str)
        } else {
            char_str
        };
        *i += 1;
        Ok(result)
    }
}

/// Check if a character is special in regex (needs escaping)
fn is_regex_special(c: &char) -> bool {
    matches!(
        c,
        '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' | '\\'
    )
}

/// Process single braces {placeholder} -> regex pattern
fn process_single_brace(chars: &[char], i: &mut usize) -> Result<String, ZervError> {
    let end_pos = find_matching_single_brace(chars, *i + 1)?;
    let placeholder: String = chars[*i + 1..end_pos].iter().collect();
    let regex_pattern = process_placeholder(&placeholder)?;
    *i = end_pos + 1; // Skip past }
    Ok(regex_pattern)
}

/// Find the end position of matching single brace } starting from start_pos
/// Handles escaped braces correctly by skipping escaped closing braces
fn find_matching_single_brace(chars: &[char], start_pos: usize) -> Result<usize, ZervError> {
    let mut j = start_pos;
    while j < chars.len() {
        if chars[j] == '}' {
            // Check if this } is escaped (preceded by \)
            if j > 0 && chars[j - 1] == '\\' {
                // This is an escaped }, skip it and continue
                j += 1;
            } else {
                // Found the matching closing brace
                return Ok(j);
            }
        } else {
            j += 1;
        }
    }

    Err(ZervError::InvalidVersion(
        "Unclosed placeholder brace".to_string(),
    ))
}

/// Process placeholder content and return corresponding regex pattern
fn process_placeholder(placeholder: &str) -> Result<String, ZervError> {
    if placeholder.starts_with("hex") {
        process_hex_placeholder(placeholder)
    } else if placeholder == "timestamp" {
        Ok(r"\d+".to_string()) // Match any sequence of digits for timestamps
    } else if let Some(regex_pattern) = placeholder.strip_prefix("regex:") {
        Ok(regex_pattern.to_string())
    } else if placeholder.is_empty() {
        Ok("".to_string())
    } else {
        Err(ZervError::InvalidVersion(format!(
            "Unknown placeholder: {{{}}}. Supported placeholders: {{hex}}, {{hex:number}}, {{timestamp}}, {{regex:pattern}}, {{}} for empty, and {{{{}}}} for literal braces",
            placeholder
        )))
    }
}

/// Process hex placeholders: {hex:length} or {hex}
fn process_hex_placeholder(placeholder: &str) -> Result<String, ZervError> {
    if placeholder == "hex" {
        Ok("[a-f0-9]+".to_string())
    } else if let Some(length_str) = placeholder.strip_prefix("hex:") {
        let length = parse_hex_length(length_str, placeholder)?;
        Ok(format!("[a-f0-9]{{{}}}", length))
    } else {
        Err(ZervError::InvalidVersion(format!(
            "Invalid hex placeholder format: {{{}}}. Expected: {{hex}} or {{hex:number}}",
            placeholder
        )))
    }
}

/// Parse and validate hex length from string
fn parse_hex_length(length_str: &str, placeholder: &str) -> Result<usize, ZervError> {
    match length_str.parse::<usize>() {
        Ok(length) => {
            if length > 0 {
                Ok(length)
            } else {
                Err(ZervError::InvalidVersion(format!(
                    "Hex length must be positive, got 0 in placeholder {{{}}}",
                    placeholder
                )))
            }
        }
        Err(_) => Err(ZervError::InvalidVersion(format!(
            "Invalid hex length in placeholder {{{}}}. Expected format: {{hex}} or {{hex:number}}",
            placeholder
        ))),
    }
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
    // Special characters (non-regex special characters are not escaped)
    #[case("version-1.0.0+test.{hex:7}", "version-1.0.0+test.a1b2c3d")]
    #[case("build[1.0.0]{hex:7}", "build[1.0.0]abc1234")]
    #[case("file+name=1.0.0.{hex:7}", "file+name=1.0.0.d4738bb")]
    // Literal braces using regex escaping
    #[case("{regex:\\{release\\}}-{hex:7}", "{release}-deadbee")]
    #[case("{regex:\\{1\\.0\\.0\\}-build}-{hex:12}", "{1.0.0}-build-cafedeadbeef")]
    #[case(
        "{regex:\\{\\{\\{\\{literal\\}\\}\\}\\}}-{hex:8}",
        "{{{{literal}}}}-a1b2c3d4"
    )]
    #[case("{regex:path\\\\to\\\\file}-{hex:7}", "path\\to\\file-abc1234")]
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
    fn test_parse_readable_pattern_no_placeholders() {
        let result = parse_readable_pattern("exact-match-1.0.0+build");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("exact-match-1.0.0+build"));
        assert!(!regex.is_match("exact-match-1.0.0+Build")); // case sensitive
        assert!(!regex.is_match("exact-match-1.0.0+build-123")); // extra characters
    }

    #[test]
    fn test_parse_readable_pattern_empty_placeholders() {
        let result = parse_readable_pattern("1.0.0-{}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("1.0.0-"));
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
    fn test_regex_escaping_for_literal_braces() {
        // Test literal braces using regex escaping
        let result = parse_readable_pattern("{regex:\\{release\\}}-{hex:7}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("{release}-deadbee"));
        assert!(!regex.is_match("release-deadbeef")); // missing literal braces

        // Test multiple literal braces with regex
        let result = parse_readable_pattern("{regex:\\{\\{\\{\\{literal\\}\\}\\}\\}}-{hex:8}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("{{{{literal}}}}-a1b2c3d4"));
        assert!(!regex.is_match("{{literal}}-a1b2c3d4")); // missing outer braces

        // Test mix of literal and regex patterns (simplified to avoid escaping complexity)
        let result = parse_readable_pattern("{regex:\\{branch\\}-[a-z]+\\d+}-{hex:12}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("{branch}-alpha123-cafedeadbeef"));
        assert!(!regex.is_match("branch-alpha123")); // missing literal braces

        // Test literal backslash with regex
        let result = parse_readable_pattern("{regex:path\\\\to\\\\file}-{hex:7}");
        assert!(result.is_ok());
        let regex = result.unwrap();
        assert!(regex.is_match("path\\to\\file-abc1234"));
        assert!(!regex.is_match("pathtofile-abc1234")); // missing backslashes
    }
}
