use std::collections::hash_map::DefaultHasher;
use std::hash::{
    Hash,
    Hasher,
};

use tera::{
    Kwargs,
    State,
    Tera,
    TeraResult,
    Value,
};

use crate::error::ZervError;
use crate::utils::sanitize::Sanitizer;

/// Timestamp format patterns
mod timestamp_patterns {
    pub const COMPACT_DATE: &str = "compact_date";
    pub const COMPACT_DATETIME: &str = "compact_datetime";
}

/// Register custom Tera functions and filters
pub fn register_functions(tera: &mut Tera) -> Result<(), ZervError> {
    tera.register_function("sanitize", sanitize_function);
    tera.register_function("hash", hash_function);
    tera.register_function("hash_int", hash_int_function);
    tera.register_function("prefix", prefix_function);
    tera.register_function("prefix_if", prefix_if_function);
    tera.register_function("format_timestamp", format_timestamp_function);
    // Override tera's built-in `default` filter. Tera 2.0 only fires `default`
    // for Undefined values, but this project serializes absent optional fields
    // as null (None), so we also fire on None to preserve the pre-2.0 behavior
    // where `{{ post | default(value=0) }}` yields "0" when post is absent.
    tera.register_filter("default", default_filter);
    Ok(())
}

/// Like tera's built-in `default`, but also substitutes the default when the
/// value is null (None), matching tera 1.x semantics.
fn default_filter(val: Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    let default_val: Value = kwargs.must_get("value")?;
    let boolean = kwargs.get::<bool>("boolean")?.unwrap_or_default();

    if boolean {
        if val.is_truthy() {
            Ok(val)
        } else {
            Ok(default_val)
        }
    } else if val.is_undefined() || val.is_none() {
        Ok(default_val)
    } else {
        Ok(val)
    }
}

/// Sanitize string with presets or custom parameters
/// Usage: {{ sanitize(value, preset="dotted") }} or {{ sanitize(value, separator="-", lowercase=true) }}
fn sanitize_function(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let value: String = kwargs.must_get("value")?;

    let preset: Option<String> = kwargs.get("preset")?;
    let separator: Option<String> = kwargs.get("separator")?;
    let keep_zeros: Option<bool> = kwargs.get("keep_zeros")?;
    let max_length: Option<u64> = kwargs.get("max_length")?;
    let lowercase: Option<bool> = kwargs.get("lowercase")?;

    let has_custom_params =
        separator.is_some() || keep_zeros.is_some() || max_length.is_some() || lowercase.is_some();

    if preset.is_some() && has_custom_params {
        return Err(tera::Error::message(
            "Cannot use preset format with custom parameters",
        ));
    }

    let sanitized = if let Some(preset) = preset {
        match preset.as_str() {
            "semver_str" | "semver" | "dotted" => Sanitizer::semver_str().sanitize(&value),
            "pep440_local_str" | "pep440" | "lower_dotted" => {
                Sanitizer::pep440_local_str().sanitize(&value)
            }
            "uint" => Sanitizer::uint().sanitize(&value),
            _ => {
                return Err(tera::Error::message(format!(
                    "Unknown sanitize preset: {}",
                    preset
                )));
            }
        }
    } else if has_custom_params {
        let sanitizer = Sanitizer::str(
            separator.as_deref(),
            lowercase.unwrap_or(false),
            keep_zeros.unwrap_or(false),
            max_length.map(|l| l as usize),
        );
        sanitizer.sanitize(&value)
    } else {
        Sanitizer::semver_str().sanitize(&value)
    };

    Ok(Value::from(sanitized))
}

/// Generate hex hash of string with configurable length
/// Usage: {{ hash(value, length=7) }}
fn hash_function(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let input: String = kwargs.must_get("value")?;
    let length = kwargs.get::<u64>("length")?.unwrap_or(7) as usize;

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());

    let short = if hash.len() > length {
        &hash[..length]
    } else {
        &hash
    };

    Ok(Value::from(short.to_string()))
}

/// Generate numeric hash with configurable length and leading zero options
/// Usage: {{ hash_int(value, length=7, allow_leading_zero=false) }}
fn hash_int_function(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let input: String = kwargs.must_get("value")?;
    let length = kwargs.get::<u64>("length")?.unwrap_or(7) as usize;
    let allow_leading_zero = kwargs.get::<bool>("allow_leading_zero")?.unwrap_or(false);

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

    Ok(Value::from(short.to_string()))
}

/// Extract prefix from string with configurable length
/// Usage: {{ prefix(value, length=10) }}
fn prefix_function(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let input: String = kwargs.must_get("value")?;
    let length = kwargs.get::<u64>("length")?.unwrap_or(10) as usize;

    let prefix = if input.len() > length {
        &input[..length]
    } else {
        &input
    };

    Ok(Value::from(prefix.to_string()))
}

/// Add conditional prefix to string (only if string is not empty)
/// Usage: {{ prefix_if(value, prefix="+") }}
fn prefix_if_function(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let value: String = kwargs.must_get("value")?;
    let prefix: String = kwargs
        .get("prefix")?
        .ok_or_else(|| tera::Error::message("prefix_if function requires a 'prefix' parameter"))?;

    if value.is_empty() {
        Ok(Value::from(""))
    } else {
        Ok(Value::from(format!("{}{}", prefix, value)))
    }
}

/// Format timestamp with customizable format
/// Usage: {{ format_timestamp(value=timestamp, format="%Y-%m-%d") }}
fn format_timestamp_function(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let timestamp: u64 = kwargs.get("value")?.ok_or_else(|| {
        tera::Error::message("format_timestamp function requires a 'value' parameter")
    })?;

    let format: String = kwargs
        .get("format")?
        .unwrap_or_else(|| "%Y-%m-%d".to_string());

    let chrono_format = match format.as_str() {
        timestamp_patterns::COMPACT_DATE => "%Y%m%d",
        timestamp_patterns::COMPACT_DATETIME => "%Y%m%d%H%M%S",
        _ => &format,
    };

    use chrono::{
        DateTime,
        Utc,
    };

    let dt = DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or_else(|| tera::Error::message("Invalid timestamp"))?
        .with_timezone(&Utc);
    let formatted = dt.format(chrono_format).to_string();

    Ok(Value::from(formatted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_functions() {
        let mut tera = Tera::default();
        let result = register_functions(&mut tera);
        assert!(result.is_ok());
    }

    fn render(template_str: &str, kwargs_json: serde_json::Value) -> String {
        let mut tera = Tera::default();
        register_functions(&mut tera).unwrap();
        tera.add_raw_template("t", template_str).unwrap();
        let ctx = tera::Context::from_serialize(&kwargs_json).unwrap();
        tera.render("t", &ctx).unwrap()
    }

    #[test]
    fn test_sanitize_function_dotted_preset() {
        let result = render(
            r#"{{ sanitize(value=branch, preset="dotted") }}"#,
            serde_json::json!({"branch": "feature-test-branch"}),
        );
        assert_eq!(result, "feature.test.branch");
    }

    #[test]
    fn test_sanitize_function_custom_params() {
        let result = render(
            r#"{{ sanitize(value=branch, separator="-", lowercase=true) }}"#,
            serde_json::json!({"branch": "feature-test-branch"}),
        );
        assert_eq!(result, "feature-test-branch");
    }

    #[test]
    fn test_sanitize_function_default() {
        let result = render(
            r#"{{ sanitize(value=branch) }}"#,
            serde_json::json!({"branch": "feature-test-branch"}),
        );
        assert_eq!(result, "feature.test.branch");
    }

    #[test]
    fn test_hash_function_default_length() {
        let result = render(
            r#"{{ hash(value=input) }}"#,
            serde_json::json!({"input": "test-input"}),
        );
        assert_eq!(result.len(), 7);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_function_custom_length() {
        let result = render(
            r#"{{ hash(value=input, length=5) }}"#,
            serde_json::json!({"input": "test-input"}),
        );
        assert_eq!(result.len(), 5);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_prefix_function_default() {
        let result = render(
            r#"{{ prefix(value=branch) }}"#,
            serde_json::json!({"branch": "feature-branch-name"}),
        );
        assert_eq!(result, "feature-br");
    }

    #[test]
    fn test_prefix_function_custom_length() {
        let result = render(
            r#"{{ prefix(value=branch, length=8) }}"#,
            serde_json::json!({"branch": "short"}),
        );
        assert_eq!(result, "short");
    }

    #[test]
    fn test_prefix_function_long_input() {
        let result = render(
            r#"{{ prefix(value=branch, length=3) }}"#,
            serde_json::json!({"branch": "very-long-branch-name"}),
        );
        assert_eq!(result, "ver");
    }

    #[test]
    fn test_prefix_if_function_with_value() {
        let result = render(
            r#"{{ prefix_if(value=v, prefix="-") }}"#,
            serde_json::json!({"v": "alpha.1"}),
        );
        assert_eq!(result, "-alpha.1");
    }

    #[test]
    fn test_prefix_if_function_with_empty_value() {
        let result = render(
            r#"{{ prefix_if(value=v, prefix="-") }}"#,
            serde_json::json!({"v": ""}),
        );
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_timestamp_function_default() {
        let result = render(
            r#"{{ format_timestamp(value=ts) }}"#,
            serde_json::json!({"ts": 1698675600u64}),
        );
        assert!(result.contains("2023-10-30"));
    }

    #[test]
    fn test_format_timestamp_function_custom() {
        let result = render(
            r#"{{ format_timestamp(value=ts, format="%Y-%m-%d") }}"#,
            serde_json::json!({"ts": 1698675600u64}),
        );
        assert_eq!(result, "2023-10-30");
    }
}
