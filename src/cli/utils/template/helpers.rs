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

fn sanitize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "sanitize helper requires a string parameter".to_string(),
        ))
    })?;

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
    let input = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "hash helper requires a string parameter".to_string(),
        ))
    })?;

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
    let input = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "hash_int helper requires a string parameter".to_string(),
        ))
    })?;

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
    let string = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "prefix helper requires a string parameter".to_string(),
        ))
    })?;

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
    let timestamp = h.param(0).and_then(|v| v.value().as_u64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "format_timestamp helper requires a timestamp parameter".to_string(),
        ))
    })?;

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
    let a = h.param(0).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "add helper requires two numeric parameters".to_string(),
        ))
    })?;

    let b = h.param(1).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "add helper requires two numeric parameters".to_string(),
        ))
    })?;

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
    let a = h.param(0).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "subtract helper requires two numeric parameters".to_string(),
        ))
    })?;

    let b = h.param(1).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "subtract helper requires two numeric parameters".to_string(),
        ))
    })?;

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
    let a = h.param(0).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "multiply helper requires two numeric parameters".to_string(),
        ))
    })?;

    let b = h.param(1).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(RenderErrorReason::Other(
            "multiply helper requires two numeric parameters".to_string(),
        ))
    })?;

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
}
