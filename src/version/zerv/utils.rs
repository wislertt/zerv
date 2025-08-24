use crate::error::{Result, ZervError};
use crate::version::zerv::{Component, PreReleaseLabel, Zerv};

pub fn extract_core_values(zerv: &Zerv) -> Vec<u64> {
    let mut core_values = Vec::new();
    for comp in &zerv.schema.core {
        let val = match comp {
            Component::VarField(field) => match field.as_str() {
                "major" => zerv.vars.major.unwrap_or(0),
                "minor" => zerv.vars.minor.unwrap_or(0),
                "patch" => zerv.vars.patch.unwrap_or(0),
                _ => 0,
            },
            Component::VarTimestamp(pattern) => {
                resolve_timestamp(pattern, zerv.vars.tag_timestamp).unwrap_or(0)
            }
            Component::Integer(n) => *n,
            _ => 0,
        };
        core_values.push(val);
    }
    core_values
}

pub fn normalize_pre_release_label(label: &str) -> Option<PreReleaseLabel> {
    match label.to_lowercase().as_str() {
        "alpha" | "a" => Some(PreReleaseLabel::Alpha),
        "beta" | "b" => Some(PreReleaseLabel::Beta),
        "rc" | "c" | "preview" | "pre" => Some(PreReleaseLabel::Rc),
        _ => None,
    }
}

fn parse_timestamp_component(
    dt: &chrono::DateTime<chrono::Utc>,
    format_str: &str,
    component_type: &str,
) -> Result<u64> {
    dt.format(format_str)
        .to_string()
        .parse()
        .map_err(|_| ZervError::InvalidFormat(format!("Failed to parse {component_type}")))
}

pub fn resolve_timestamp(pattern: &str, timestamp: Option<u64>) -> Result<u64> {
    let ts = timestamp.ok_or_else(|| {
        ZervError::InvalidFormat("Timestamp is required but was None".to_string())
    })?;
    let dt = chrono::DateTime::from_timestamp(ts as i64, 0)
        .ok_or_else(|| ZervError::InvalidFormat("Invalid timestamp".to_string()))?;

    let result = match pattern {
        "YYYY" => parse_timestamp_component(&dt, "%Y", "year")?,
        "YY" => parse_timestamp_component(&dt, "%y", "year")?,
        "MM" => parse_timestamp_component(&dt, "%-m", "month")?,
        "0M" => parse_timestamp_component(&dt, "%m", "month")?,
        "WW" => parse_timestamp_component(&dt, "%-W", "week")?,
        "0W" => parse_timestamp_component(&dt, "%W", "week")?,
        "DD" => parse_timestamp_component(&dt, "%-d", "day")?,
        "0D" => parse_timestamp_component(&dt, "%d", "day")?,
        "HH" => parse_timestamp_component(&dt, "%-H", "hour")?,
        "0H" => parse_timestamp_component(&dt, "%H", "hour")?,
        "mm" => parse_timestamp_component(&dt, "%-M", "minute")?,
        "0m" => parse_timestamp_component(&dt, "%M", "minute")?,
        "SS" => parse_timestamp_component(&dt, "%-S", "second")?,
        "0S" => parse_timestamp_component(&dt, "%S", "second")?,
        _ => {
            return Err(ZervError::InvalidFormat(format!(
                "Unknown timestamp pattern: {pattern}"
            )));
        }
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_resolve_timestamp_none() {
        assert_eq!(
            resolve_timestamp("YYYY", None),
            Err(ZervError::InvalidFormat(
                "Timestamp is required but was None".to_string()
            ))
        );
    }

    #[rstest]
    #[case(1710511845, "YYYY", 2024)] // 2024-03-15 14:10:45
    #[case(1710511845, "YY", 24)]
    #[case(1710511845, "MM", 3)]
    #[case(1710511845, "0M", 3)]
    #[case(1710511845, "WW", 11)]
    #[case(1710511845, "0W", 11)]
    #[case(1710511845, "DD", 15)]
    #[case(1710511845, "0D", 15)]
    #[case(1710511845, "HH", 14)]
    #[case(1710511845, "0H", 14)]
    #[case(1710511845, "mm", 10)]
    #[case(1710511845, "0m", 10)]
    #[case(1710511845, "SS", 45)]
    #[case(1710511845, "0S", 45)]
    #[case(1577836800, "YYYY", 2020)] // 2020-01-01 00:00:00 - test leading zeros
    #[case(1577836800, "MM", 1)]
    #[case(1577836800, "0M", 1)]
    #[case(1577836800, "DD", 1)]
    #[case(1577836800, "0D", 1)]
    #[case(1577836800, "WW", 0)]
    #[case(1577836800, "0W", 0)]
    #[case(1577836800, "HH", 0)]
    #[case(1577836800, "0H", 0)]
    #[case(1577836800, "mm", 0)]
    #[case(1577836800, "0m", 0)]
    #[case(1577836800, "SS", 0)]
    #[case(1577836800, "0S", 0)]
    #[case(1609459200, "MM", 1)] // 2021-01-01 00:00:00 - different year
    #[case(1609459200, "0M", 1)]
    #[case(1609459200, "WW", 0)]
    #[case(1609459200, "0W", 0)]
    fn test_resolve_timestamp_patterns(
        #[case] timestamp: u64,
        #[case] pattern: &str,
        #[case] expected: u64,
    ) {
        assert_eq!(
            resolve_timestamp(pattern, Some(timestamp)).unwrap_or_else(|_| panic!(
                "Failed to resolve timestamp pattern '{pattern}' for timestamp {timestamp}"
            )),
            expected
        );
    }

    #[test]
    fn test_resolve_timestamp_unknown_pattern() {
        let timestamp = Some(1710511845);
        assert_eq!(
            resolve_timestamp("INVALID", timestamp),
            Err(ZervError::InvalidFormat(
                "Unknown timestamp pattern: INVALID".to_string()
            ))
        );
    }
}
