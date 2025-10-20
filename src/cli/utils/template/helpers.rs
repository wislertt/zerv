use std::collections::hash_map::DefaultHasher;
use std::hash::{
    Hash,
    Hasher,
};

use handlebars::{
    Context,
    Handlebars,
    Helper,
    HelperResult,
    Output,
    RenderContext,
    RenderErrorReason,
};

use crate::error::ZervError;
use crate::utils::constants::timestamp_patterns;
use crate::utils::sanitize::Sanitizer;

/// Register custom Zerv helpers for Handlebars
pub fn register_helpers(handlebars: &mut Handlebars) -> Result<(), ZervError> {
    handlebars.register_helper("sanitize", Box::new(sanitize_helper));
    handlebars.register_helper("hash", Box::new(hash_helper));
    handlebars.register_helper("hash_int", Box::new(hash_int_helper));
    handlebars.register_helper("prefix", Box::new(prefix_helper));
    handlebars.register_helper("format_timestamp", Box::new(format_timestamp_helper));
    handlebars.register_helper("add", Box::new(add_helper));
    handlebars.register_helper("subtract", Box::new(subtract_helper));
    handlebars.register_helper("multiply", Box::new(multiply_helper));
    Ok(())
}

// ============================================================================
// Parameter Extraction Helpers
// ============================================================================

/// Extract string parameter from a helper
fn extract_string_param<'a>(
    h: &'a Helper,
    helper_name: &str,
) -> Result<&'a str, handlebars::RenderError> {
    h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(format!(
            "{helper_name} helper requires a string parameter"
        )))
    })
}

/// Extract u64 parameter from a helper
fn extract_u64_param(h: &Helper, helper_name: &str) -> Result<u64, handlebars::RenderError> {
    h.param(0).and_then(|v| v.value().as_u64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(format!(
            "{helper_name} helper requires a numeric parameter"
        )))
    })
}

/// Extract two numeric parameters from a helper
fn extract_two_numbers(
    h: &Helper,
    helper_name: &str,
) -> Result<(i64, i64), handlebars::RenderError> {
    let a = h.param(0).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(format!(
            "{helper_name} helper requires two numeric parameters"
        )))
    })?;

    let b = h.param(1).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(format!(
            "{helper_name} helper requires two numeric parameters"
        )))
    })?;

    Ok((a, b))
}

// ============================================================================
// Helper Implementations
// ============================================================================

fn sanitize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = extract_string_param(h, "sanitize")?;

    // Check for preset format
    let format = h.hash_get("preset").and_then(|v| v.value().as_str());

    // Check for custom parameters
    let separator = h.hash_get("separator").and_then(|v| v.value().as_str());
    let keep_zeros = h.hash_get("keep_zeros").and_then(|v| v.value().as_bool());
    let max_length = h.hash_get("max_length").and_then(|v| v.value().as_u64());
    let lowercase = h.hash_get("lowercase").and_then(|v| v.value().as_bool());

    let has_custom_params =
        separator.is_some() || keep_zeros.is_some() || max_length.is_some() || lowercase.is_some();

    // Error if both format and custom parameters are specified
    if format.is_some() && has_custom_params {
        return Err(handlebars::RenderError::from(RenderErrorReason::Other(
            "Cannot use preset format with custom parameters".to_string(),
        )));
    }

    let sanitized = if let Some(fmt) = format {
        // Use preset format
        match fmt {
            "semver_str" | "semver" | "dotted" => Sanitizer::semver_str().sanitize(value),
            "pep440_local_str" | "pep440" | "lower_dotted" => {
                Sanitizer::pep440_local_str().sanitize(value)
            }
            "uint" => Sanitizer::uint().sanitize(value),
            _ => {
                return Err(handlebars::RenderError::from(RenderErrorReason::Other(
                    format!("Unknown sanitize preset: {fmt}"),
                )));
            }
        }
    } else if has_custom_params {
        // Use custom parameters
        let sanitizer = Sanitizer::str(
            separator,
            lowercase.unwrap_or(false),
            keep_zeros.unwrap_or(false),
            max_length.map(|l| l as usize),
        );
        sanitizer.sanitize(value)
    } else {
        // Default to pep440_local_str
        Sanitizer::pep440_local_str().sanitize(value)
    };

    out.write(&sanitized)?;
    Ok(())
}

/// Generate hex hash from input (default: 7 chars)
fn hash_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let input = extract_string_param(h, "hash")?;

    let length = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(7) as usize;

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());

    let short = if hash.len() > length {
        &hash[..length]
    } else {
        &hash
    };

    out.write(short)?;
    Ok(())
}

fn format_with_leading_zeros(hash_num: u64, length: usize) -> String {
    let hash_str = hash_num.to_string();
    if hash_str.len() > length {
        hash_str[..length].to_string()
    } else if length >= 20 {
        format!("{hash_num:0length$}")
    } else {
        format!(
            "{:0width$}",
            hash_num % 10_u64.pow(length as u32),
            width = length
        )
    }
}

fn format_without_leading_zeros(hash_num: u64, length: usize) -> String {
    if length == 0 {
        return "0".to_string();
    }

    if length == 20 {
        let hash_str = hash_num.to_string();
        if hash_str.len() >= 20 {
            return hash_str[..20].to_string();
        }
        let padded = format!("{hash_num:020}");
        if let Some(stripped) = padded.strip_prefix('0') {
            format!("1{stripped}")
        } else {
            padded
        }
    } else {
        let min_val = 10_u64.pow((length - 1) as u32);
        let max_val = 10_u64.pow(length as u32) - 1;
        let range = max_val - min_val + 1;
        (hash_num % range + min_val).to_string()
    }
}

/// Generate integer hash from input
fn hash_int_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let input = extract_string_param(h, "hash_int")?;

    let length = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(7) as usize;
    let allow_leading_zero = h
        .hash_get("allow_leading_zero")
        .and_then(|v| v.value().as_bool())
        .unwrap_or(false);

    // Validate length limits to prevent overflow
    if length > 20 {
        return Err(handlebars::RenderError::from(RenderErrorReason::Other(
            "hash_int length must be 20 or less".to_string(),
        )));
    }

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash_num = hasher.finish();

    let result = if allow_leading_zero {
        format_with_leading_zeros(hash_num, length)
    } else {
        format_without_leading_zeros(hash_num, length)
    };

    out.write(&result)?;
    Ok(())
}

/// Get prefix of string to length
fn prefix_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let string = extract_string_param(h, "prefix")?;

    let length = h
        .param(1)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(string.len() as u64) as usize;

    let prefix = if string.len() > length {
        &string[..length]
    } else {
        string
    };

    out.write(prefix)?;
    Ok(())
}

/// Format unix timestamp to string
fn format_timestamp_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let timestamp = extract_u64_param(h, "format_timestamp")?;

    let format = h
        .hash_get("format")
        .and_then(|v| v.value().as_str())
        .unwrap_or("%Y-%m-%d");

    let chrono_format = match format {
        timestamp_patterns::COMPACT_DATE => "%Y%m%d",
        timestamp_patterns::COMPACT_DATETIME => "%Y%m%d%H%M%S",
        _ => format,
    };

    let dt = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or_else(|| {
            handlebars::RenderError::from(RenderErrorReason::Other("Invalid timestamp".to_string()))
        })?
        .naive_utc();

    let formatted = dt.format(chrono_format).to_string();
    out.write(&formatted)?;
    Ok(())
}

/// Add two numbers
fn add_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let (a, b) = extract_two_numbers(h, "add")?;
    out.write(&(a + b).to_string())?;
    Ok(())
}

/// Subtract two numbers
fn subtract_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let (a, b) = extract_two_numbers(h, "subtract")?;
    out.write(&(a - b).to_string())?;
    Ok(())
}

/// Multiply two numbers
fn multiply_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let (a, b) = extract_two_numbers(h, "multiply")?;
    out.write(&(a * b).to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use handlebars::Handlebars;
    use rstest::rstest;

    use super::*;

    fn render_template(template: &str) -> String {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();
        hb.render_template(template, &()).unwrap()
    }

    #[rstest]
    // Preset formats
    #[case("{{sanitize 'Feature/API-v2' preset='dotted'}}", "Feature.API.v2")]
    #[case("{{sanitize 'Build-ID-0051' preset='semver'}}", "Build.ID.51")]
    #[case(
        "{{sanitize 'Feature/API-v2' preset='lower_dotted'}}",
        "feature.api.v2"
    )]
    #[case("{{sanitize 'Build-ID-0051' preset='pep440'}}", "build.id.51")]
    #[case("{{sanitize '0051' preset='uint'}}", "51")]
    #[case("{{sanitize 'abc123' preset='uint'}}", "")]
    #[case("{{sanitize 'Feature/API-v2'}}", "feature.api.v2")]
    // Custom parameters
    #[case("{{sanitize 'Feature-API' separator='_'}}", "Feature_API")]
    #[case(
        "{{sanitize 'Feature-API' separator='_' lowercase=true}}",
        "feature_api"
    )]
    #[case(
        "{{sanitize 'test-0051-build' separator='.' keep_zeros=true lowercase=false}}",
        "test.0051.build"
    )]
    #[case(
        "{{sanitize 'test-0051-build' separator='.' keep_zeros=false lowercase=false}}",
        "test.51.build"
    )]
    #[case(
        "{{sanitize 'VeryLongBranchName' max_length=10 lowercase=false}}",
        "VeryLongBr"
    )]
    #[case(
        "{{sanitize 'Test-Branch' separator='-' lowercase=false}}",
        "Test-Branch"
    )]
    #[case(
        "{{sanitize 'feature/test' separator='' lowercase=true}}",
        "featuretest"
    )]
    fn test_sanitize_helper(#[case] template: &str, #[case] expected: &str) {
        assert_eq!(render_template(template), expected);
    }

    #[test]
    fn test_sanitize_helper_errors() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        // Test conflict between preset and custom parameters
        let result = hb.render_template("{{sanitize 'test' preset='dotted' separator='_'}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot use preset format with custom parameters")
        );

        // Test unknown preset
        let result = hb.render_template("{{sanitize 'test' preset='unknown_preset'}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown sanitize preset: unknown_preset")
        );
    }

    #[rstest]
    #[case("{{hash 'test'}}", "c7dedb4")]
    #[case("{{hash 'test' 10}}", "c7dedb4632")]
    #[case("{{hash_int 'test'}}", "7126668")]
    #[case("{{hash_int 'test' 5 allow_leading_zero=false}}", "16668")]
    #[case("{{hash_int 'test' 5 allow_leading_zero=true}}", "14402")]
    #[case("{{prefix 'abcdef123456789' 7}}", "abcdef1")]
    #[case("{{prefix 'abc' 10}}", "abc")]
    #[case("{{format_timestamp 1703123456 format='%Y-%m-%d'}}", "2023-12-21")]
    #[case("{{format_timestamp 1703123456 format='compact_date'}}", "20231221")]
    #[case(
        "{{format_timestamp 1703123456 format='compact_datetime'}}",
        "20231221015056"
    )]
    #[case("{{add 5 3}}", "8")]
    #[case("{{subtract 10 4}}", "6")]
    #[case("{{multiply 7 6}}", "42")]
    fn test_helpers(#[case] template: &str, #[case] expected: &str) {
        assert_eq!(render_template(template), expected);
    }

    fn assert_hash_int_no_leading_zero(input: &str, length: usize) {
        let result = render_template(&format!(
            "{{{{hash_int '{input}' {length} allow_leading_zero=false}}}}"
        ));
        assert_eq!(result.len(), length);
        if length > 1 {
            assert!(!result.starts_with('0'));
        }
        let num: u64 = result.parse().unwrap();
        let min_val = 10_u64.pow((length - 1) as u32);
        let max_val = 10_u64.pow(length as u32) - 1;
        assert!(num >= min_val && num <= max_val);
    }

    fn assert_hash_int_with_leading_zero(input: &str, length: usize) {
        let result = render_template(&format!(
            "{{{{hash_int '{input}' {length} allow_leading_zero=true}}}}"
        ));
        assert!(result.len() <= length);
    }

    #[test]
    fn test_hash_int_length_limits() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        // Test length > 20 should error for both cases
        let result = hb.render_template("{{hash_int 'test' 21 allow_leading_zero=false}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("length must be 20 or less")
        );

        let result = hb.render_template("{{hash_int 'test' 21 allow_leading_zero=true}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("length must be 20 or less")
        );

        // Test length = 20 should work for both cases
        let result = hb.render_template("{{hash_int 'test' 20 allow_leading_zero=false}}", &());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 20);

        let result = hb.render_template("{{hash_int 'test' 20 allow_leading_zero=true}}", &());
        assert!(result.is_ok());
        assert!(result.unwrap().len() <= 20);
    }

    #[test]
    fn test_hash_int_digit_guarantees() {
        let chars: Vec<char> = ('0'..='9').chain('a'..='z').collect();
        let inputs: Vec<String> = chars
            .iter()
            .flat_map(|&a| chars.iter().map(move |&b| format!("{a}{b}")))
            .collect();

        for input in inputs {
            for length in 1..=5 {
                assert_hash_int_no_leading_zero(&input, length);
                assert_hash_int_with_leading_zero(&input, length);
            }
        }
    }

    #[rstest]
    #[case("{{add 5 3}}", "8")]
    #[case("{{add 0 0}}", "0")]
    #[case("{{add -5 3}}", "-2")]
    #[case("{{subtract 10 4}}", "6")]
    #[case("{{subtract 0 5}}", "-5")]
    #[case("{{subtract -3 -7}}", "4")]
    #[case("{{multiply 7 6}}", "42")]
    #[case("{{multiply 0 100}}", "0")]
    #[case("{{multiply -4 3}}", "-12")]
    fn test_math_helpers(#[case] template: &str, #[case] expected: &str) {
        assert_eq!(render_template(template), expected);
    }

    #[test]
    fn test_sanitize_missing_parameter() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{sanitize}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires a string parameter")
        );
    }

    #[test]
    fn test_hash_missing_parameter() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{hash}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires a string parameter")
        );
    }

    #[test]
    fn test_hash_int_missing_parameter() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{hash_int}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires a string parameter")
        );
    }

    #[test]
    fn test_prefix_missing_parameter() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{prefix}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires a string parameter")
        );
    }

    #[test]
    fn test_format_timestamp_missing_parameter() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{format_timestamp}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires a numeric parameter")
        );
    }

    #[test]
    fn test_format_timestamp_invalid_timestamp() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        // Test with an extremely large timestamp that would be invalid
        let result = hb.render_template("{{format_timestamp 99999999999999999}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid timestamp")
        );
    }

    #[test]
    fn test_add_missing_parameters() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        // Missing first parameter
        let result = hb.render_template("{{add}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires two numeric parameters")
        );

        // Missing second parameter
        let result = hb.render_template("{{add 5}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires two numeric parameters")
        );
    }

    #[test]
    fn test_subtract_missing_parameters() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{subtract}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires two numeric parameters")
        );

        let result = hb.render_template("{{subtract 10}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires two numeric parameters")
        );
    }

    #[test]
    fn test_multiply_missing_parameters() {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{multiply}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires two numeric parameters")
        );

        let result = hb.render_template("{{multiply 7}}", &());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires two numeric parameters")
        );
    }

    #[test]
    fn test_format_with_leading_zeros_edge_cases() {
        // Test when hash_str is longer than length (20 digit number, take first 5)
        let result = format_with_leading_zeros(12345678901234567890, 5);
        assert_eq!(result.len(), 5);
        assert_eq!(result, "12345"); // First 5 chars of the string

        // Test length at boundary (exactly 20)
        let result = format_with_leading_zeros(123, 20);
        assert_eq!(result, "00000000000000000123");

        // Test length > 20
        let result = format_with_leading_zeros(123, 25);
        assert_eq!(result, "0000000000000000000000123");

        // Test length < 20 with modulo - it uses modulo to get last N digits
        let result = format_with_leading_zeros(456789, 10);
        assert_eq!(result.len(), 10);
        assert_eq!(result, "0000456789"); // 456789 padded to 10 digits

        // Test modulo case: number has more digits than length
        let result = format_with_leading_zeros(123456789, 5);
        assert_eq!(result.len(), 5);
        assert_eq!(result, "12345"); // First 5 digits (hash_str is 9 chars, > 5)

        // Test length = 1 where hash_str.len() > length (takes first char)
        let result = format_with_leading_zeros(12345, 1);
        assert_eq!(result, "1"); // First char of "12345"

        // Test length = 1 with actual modulo (need 1-digit number or use modulo path)
        let result = format_with_leading_zeros(7, 1);
        assert_eq!(result, "7"); // 7 padded to 1 digit

        // Test modulo with length < hash_str length
        let result = format_with_leading_zeros(789, 5);
        assert_eq!(result, "00789"); // 789 padded to 5 digits

        // Test another case where hash_str length > requested length
        let result = format_with_leading_zeros(9876543210987654321, 3);
        assert_eq!(result.len(), 3);
        assert_eq!(result, "987"); // First 3 chars
    }

    #[test]
    fn test_format_without_leading_zeros_edge_cases() {
        // Test length = 0
        let result = format_without_leading_zeros(12345, 0);
        assert_eq!(result, "0");

        // Test length = 20 with short hash
        let result = format_without_leading_zeros(123, 20);
        assert!(result.len() == 20);
        assert!(!result.starts_with('0'));

        // Test length = 20 with long hash
        let result = format_without_leading_zeros(12345678901234567890, 20);
        assert_eq!(result.len(), 20);

        // Test length = 1
        let result = format_without_leading_zeros(0, 1);
        assert!(result.len() == 1);

        // Test various lengths ensure no leading zeros
        for length in 2..10 {
            let result = format_without_leading_zeros(123456789, length);
            assert_eq!(result.len(), length);
            assert!(!result.starts_with('0'));
        }
    }

    #[test]
    fn test_hash_int_zero_length() {
        let result = render_template("{{hash_int 'test' 0 allow_leading_zero=false}}");
        assert_eq!(result, "0");
    }

    #[test]
    fn test_hash_int_various_lengths_with_leading_zero() {
        // Test all lengths from 1 to 20
        for length in 1..=20 {
            let result = render_template(&format!(
                "{{{{hash_int 'test_input' {length} allow_leading_zero=true}}}}"
            ));
            assert!(
                result.len() <= length,
                "Length {length} resulted in '{result}' with len {}",
                result.len()
            );
        }
    }

    #[test]
    fn test_prefix_default_length() {
        // When no length is provided, should return full string
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        let result = hb.render_template("{{prefix 'testing'}}", &()).unwrap();
        assert_eq!(result, "testing");
    }

    #[test]
    fn test_format_timestamp_default_format() {
        // Test default format when no format parameter is provided
        let result = render_template("{{format_timestamp 1703123456}}");
        assert_eq!(result, "2023-12-21");
    }

    #[test]
    fn test_hash_length_shorter_than_output() {
        // Test when hash is naturally shorter than requested length
        let mut hb = Handlebars::new();
        register_helpers(&mut hb).unwrap();

        // Create a hash and verify it handles short hashes correctly
        let result = hb.render_template("{{hash 'x' 100}}", &()).unwrap();
        // The result should be the full hash (not padded)
        assert!(result.len() <= 100);
    }

    // Tests for parameter extraction functions
    mod extract_param_tests {
        use handlebars::{
            Context,
            Handlebars,
            Helper,
            HelperResult,
            Output,
            RenderContext,
        };

        use super::super::*;

        /// Helper to create a Handlebars instance with a test helper that extracts a string param
        fn create_string_param_helper() -> Handlebars<'static> {
            let mut hb = Handlebars::new();
            hb.register_helper(
                "test",
                Box::new(
                    |h: &Helper,
                     _: &Handlebars,
                     _: &Context,
                     _: &mut RenderContext,
                     out: &mut dyn Output|
                     -> HelperResult {
                        let value = extract_string_param(h, "test")?;
                        out.write(value)?;
                        Ok(())
                    },
                ),
            );
            hb
        }

        /// Helper to create a Handlebars instance with a test helper that extracts a u64 param
        fn create_u64_param_helper() -> Handlebars<'static> {
            let mut hb = Handlebars::new();
            hb.register_helper(
                "test",
                Box::new(
                    |h: &Helper,
                     _: &Handlebars,
                     _: &Context,
                     _: &mut RenderContext,
                     out: &mut dyn Output|
                     -> HelperResult {
                        let value = extract_u64_param(h, "test")?;
                        out.write(&value.to_string())?;
                        Ok(())
                    },
                ),
            );
            hb
        }

        /// Helper to create a Handlebars instance with a test helper that extracts two numbers
        fn create_two_numbers_helper() -> Handlebars<'static> {
            let mut hb = Handlebars::new();
            hb.register_helper(
                "test",
                Box::new(
                    |h: &Helper,
                     _: &Handlebars,
                     _: &Context,
                     _: &mut RenderContext,
                     out: &mut dyn Output|
                     -> HelperResult {
                        let (a, b) = extract_two_numbers(h, "test")?;
                        out.write(&format!("{a},{b}"))?;
                        Ok(())
                    },
                ),
            );
            hb
        }

        /// Helper to assert error message contains expected text
        fn assert_error_contains(result: Result<String, handlebars::RenderError>, expected: &str) {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains(expected));
        }

        #[test]
        fn test_extract_string_param_success() {
            let hb = create_string_param_helper();
            let result = hb.render_template("{{test 'hello'}}", &()).unwrap();
            assert_eq!(result, "hello");
        }

        #[test]
        fn test_extract_string_param_missing() {
            let hb = create_string_param_helper();
            assert_error_contains(
                hb.render_template("{{test}}", &()),
                "requires a string parameter",
            );
        }

        #[test]
        fn test_extract_string_param_wrong_type() {
            let hb = create_string_param_helper();
            assert_error_contains(
                hb.render_template("{{test 123}}", &()),
                "requires a string parameter",
            );
        }

        #[test]
        fn test_extract_u64_param_success() {
            let hb = create_u64_param_helper();
            let result = hb.render_template("{{test 42}}", &()).unwrap();
            assert_eq!(result, "42");
        }

        #[test]
        fn test_extract_u64_param_missing() {
            let hb = create_u64_param_helper();
            assert_error_contains(
                hb.render_template("{{test}}", &()),
                "requires a numeric parameter",
            );
        }

        #[test]
        fn test_extract_u64_param_wrong_type() {
            let hb = create_u64_param_helper();
            assert_error_contains(
                hb.render_template("{{test 'not a number'}}", &()),
                "requires a numeric parameter",
            );
        }

        #[test]
        fn test_extract_u64_param_negative() {
            let hb = create_u64_param_helper();
            // as_u64() returns None for negative numbers
            assert_error_contains(
                hb.render_template("{{test -5}}", &()),
                "requires a numeric parameter",
            );
        }

        #[test]
        fn test_extract_two_numbers_success() {
            let hb = create_two_numbers_helper();
            let result = hb.render_template("{{test 10 20}}", &()).unwrap();
            assert_eq!(result, "10,20");
        }

        #[test]
        fn test_extract_two_numbers_negative() {
            let hb = create_two_numbers_helper();
            let result = hb.render_template("{{test -5 3}}", &()).unwrap();
            assert_eq!(result, "-5,3");
        }

        #[test]
        fn test_extract_two_numbers_missing_first() {
            let hb = create_two_numbers_helper();
            assert_error_contains(
                hb.render_template("{{test}}", &()),
                "requires two numeric parameters",
            );
        }

        #[test]
        fn test_extract_two_numbers_missing_second() {
            let hb = create_two_numbers_helper();
            assert_error_contains(
                hb.render_template("{{test 10}}", &()),
                "requires two numeric parameters",
            );
        }

        #[test]
        fn test_extract_two_numbers_wrong_type_first() {
            let hb = create_two_numbers_helper();
            assert_error_contains(
                hb.render_template("{{test 'abc' 10}}", &()),
                "requires two numeric parameters",
            );
        }

        #[test]
        fn test_extract_two_numbers_wrong_type_second() {
            let hb = create_two_numbers_helper();
            assert_error_contains(
                hb.render_template("{{test 10 'xyz'}}", &()),
                "requires two numeric parameters",
            );
        }
    }
}
