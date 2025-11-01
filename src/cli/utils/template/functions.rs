use std::collections::hash_map::DefaultHasher;
use std::hash::{
    Hash,
    Hasher,
};

use tera::{
    Tera,
    Value,
};

use crate::error::ZervError;
use crate::utils::sanitize::Sanitizer;

/// Timestamp format patterns
mod timestamp_patterns {
    pub const COMPACT_DATE: &str = "compact_date";
    pub const COMPACT_DATETIME: &str = "compact_datetime";
}

/// Register custom Tera functions
pub fn register_functions(tera: &mut Tera) -> Result<(), ZervError> {
    tera.register_function("sanitize", Box::new(sanitize_function));
    tera.register_function("hash", Box::new(hash_function));
    tera.register_function("hash_int", Box::new(hash_int_function));
    tera.register_function("prefix", Box::new(prefix_function));
    tera.register_function("format_timestamp", Box::new(format_timestamp_function));
    Ok(())
}

/// Sanitize string with presets or custom parameters
/// Usage: {{ sanitize(value, preset="dotted") }} or {{ sanitize(value, separator="-", lowercase=true) }}
fn sanitize_function(
    args: &std::collections::HashMap<String, Value>,
) -> Result<Value, tera::Error> {
    let value = args
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("sanitize function requires a 'value' parameter"))?;

    // Check for preset format
    let preset = args.get("preset").and_then(|v| v.as_str());

    // Check for custom parameters
    let separator = args.get("separator").and_then(|v| v.as_str());
    let keep_zeros = args.get("keep_zeros").and_then(|v| v.as_bool());
    let max_length = args.get("max_length").and_then(|v| v.as_u64());
    let lowercase = args.get("lowercase").and_then(|v| v.as_bool());

    let has_custom_params =
        separator.is_some() || keep_zeros.is_some() || max_length.is_some() || lowercase.is_some();

    // Error if both preset and custom parameters are specified
    if preset.is_some() && has_custom_params {
        return Err(tera::Error::msg(
            "Cannot use preset format with custom parameters",
        ));
    }

    let sanitized = if let Some(preset) = preset {
        // Use preset format
        match preset {
            "semver_str" | "semver" | "dotted" => Sanitizer::semver_str().sanitize(value),
            "pep440_local_str" | "pep440" | "lower_dotted" => {
                Sanitizer::pep440_local_str().sanitize(value)
            }
            "uint" => Sanitizer::uint().sanitize(value),
            _ => {
                return Err(tera::Error::msg(format!(
                    "Unknown sanitize preset: {}",
                    preset
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
        // Default to dotted preset if no parameters specified
        Sanitizer::semver_str().sanitize(value)
    };

    Ok(Value::String(sanitized))
}

/// Generate hex hash of string with configurable length
/// Usage: {{ hash(value, length=7) }}
fn hash_function(args: &std::collections::HashMap<String, Value>) -> Result<Value, tera::Error> {
    let input = args
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("hash function requires a 'value' parameter"))?;

    let length = args.get("length").and_then(|v| v.as_u64()).unwrap_or(7) as usize;

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());

    let short = if hash.len() > length {
        &hash[..length]
    } else {
        &hash
    };

    Ok(Value::String(short.to_string()))
}

/// Generate numeric hash with configurable length and leading zero options
/// Usage: {{ hash_int(value, length=7, allow_leading_zero=false) }}
fn hash_int_function(
    args: &std::collections::HashMap<String, Value>,
) -> Result<Value, tera::Error> {
    let input = args
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("hash_int function requires a 'value' parameter"))?;

    let length = args.get("length").and_then(|v| v.as_u64()).unwrap_or(7) as usize;

    let allow_leading_zero = args
        .get("allow_leading_zero")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    let result = if allow_leading_zero {
        format!("{:0width$}", hash, width = length)
    } else {
        format!("{}", hash)
    };

    let short = if result.len() > length {
        &result[..length]
    } else {
        &result
    };

    Ok(Value::String(short.to_string()))
}

/// Extract prefix from string with configurable length
/// Usage: {{ prefix(value, length=10) }}
fn prefix_function(args: &std::collections::HashMap<String, Value>) -> Result<Value, tera::Error> {
    let input = args
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("prefix function requires a 'value' parameter"))?;

    let length = args.get("length").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let prefix = if input.len() > length {
        &input[..length]
    } else {
        input
    };

    Ok(Value::String(prefix.to_string()))
}

/// Format timestamp with customizable format
/// Usage: {{ format_timestamp(timestamp, format="%Y-%m-%d") }}
fn format_timestamp_function(
    args: &std::collections::HashMap<String, Value>,
) -> Result<Value, tera::Error> {
    let timestamp = args.get("value").and_then(|v| v.as_u64()).ok_or_else(|| {
        tera::Error::msg("format_timestamp function requires a 'value' parameter")
    })?;

    // Default format if not specified, and handle special format names
    let format = args
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("%Y-%m-%d");

    let chrono_format = match format {
        timestamp_patterns::COMPACT_DATE => "%Y%m%d",
        timestamp_patterns::COMPACT_DATETIME => "%Y%m%d%H%M%S",
        _ => format,
    };

    // Convert timestamp to DateTime and format
    use chrono::{
        DateTime,
        Utc,
    };

    let dt = DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or_else(|| tera::Error::msg("Invalid timestamp"))?
        .with_timezone(&Utc);
    let formatted = dt.format(chrono_format).to_string();

    Ok(Value::String(formatted))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_register_functions() {
        let mut tera = Tera::default();
        let result = register_functions(&mut tera);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_function_dotted_preset() {
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::String("feature-test-branch".to_string()),
        );
        args.insert("preset".to_string(), Value::String("dotted".to_string()));

        let result = sanitize_function(&args).unwrap();
        assert_eq!(result, Value::String("feature.test.branch".to_string()));
    }

    #[test]
    fn test_sanitize_function_custom_params() {
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::String("feature-test-branch".to_string()),
        );
        args.insert("separator".to_string(), Value::String("-".to_string()));
        args.insert("lowercase".to_string(), Value::Bool(true));

        let result = sanitize_function(&args).unwrap();
        assert_eq!(result, Value::String("feature-test-branch".to_string()));
    }

    #[test]
    fn test_sanitize_function_default() {
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::String("feature-test-branch".to_string()),
        );

        let result = sanitize_function(&args).unwrap();
        assert_eq!(result, Value::String("feature.test.branch".to_string()));
    }

    #[test]
    fn test_sanitize_function_error_both_preset_and_custom() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("test".to_string()));
        args.insert("preset".to_string(), Value::String("dotted".to_string()));
        args.insert("separator".to_string(), Value::String("-".to_string()));

        let result = sanitize_function(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_function_default_length() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("test-input".to_string()));

        let result = hash_function(&args).unwrap();
        let hash_str = result.as_str().unwrap();
        assert_eq!(hash_str.len(), 7);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_function_custom_length() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("test-input".to_string()));
        args.insert("length".to_string(), Value::Number(5.into()));

        let result = hash_function(&args).unwrap();
        let hash_str = result.as_str().unwrap();
        assert_eq!(hash_str.len(), 5);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_int_function_default() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("test-input".to_string()));

        let result = hash_int_function(&args).unwrap();
        let hash_str = result.as_str().unwrap();
        assert_eq!(hash_str.len(), 7);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_int_function_leading_zeros() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("test-input".to_string()));
        args.insert("length".to_string(), Value::Number(10.into()));
        args.insert("allow_leading_zero".to_string(), Value::Bool(true));

        let result = hash_int_function(&args).unwrap();
        let hash_str = result.as_str().unwrap();
        assert_eq!(hash_str.len(), 10);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_prefix_function_default() {
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::String("feature-branch-name".to_string()),
        );

        let result = prefix_function(&args).unwrap();
        assert_eq!(result, Value::String("feature-br".to_string()));
    }

    #[test]
    fn test_prefix_function_custom_length() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("short".to_string()));
        args.insert("length".to_string(), Value::Number(8.into()));

        let result = prefix_function(&args).unwrap();
        assert_eq!(result, Value::String("short".to_string()));
    }

    #[test]
    fn test_prefix_function_long_input() {
        let mut args = HashMap::new();
        args.insert(
            "value".to_string(),
            Value::String("very-long-branch-name".to_string()),
        );
        args.insert("length".to_string(), Value::Number(3.into()));

        let result = prefix_function(&args).unwrap();
        assert_eq!(result, Value::String("ver".to_string()));
    }

    #[test]
    fn test_format_timestamp_function_default() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::Number(1698675600.into())); // 2023-10-30 12:00:00 UTC

        let result = format_timestamp_function(&args).unwrap();
        let formatted = result.as_str().unwrap();
        assert!(formatted.contains("2023-10-30"));
    }

    #[test]
    fn test_format_timestamp_function_custom() {
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::Number(1698675600.into())); // 2023-10-30 12:00:00 UTC
        args.insert("format".to_string(), Value::String("%Y-%m-%d".to_string()));

        let result = format_timestamp_function(&args).unwrap();
        assert_eq!(result, Value::String("2023-10-30".to_string()));
    }
}
