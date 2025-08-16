use crate::version::zerv::PreReleaseLabel;

pub fn normalize_pre_release_label(label: &str) -> Option<PreReleaseLabel> {
    match label.to_lowercase().as_str() {
        "alpha" | "a" => Some(PreReleaseLabel::Alpha),
        "beta" | "b" => Some(PreReleaseLabel::Beta),
        "rc" | "c" | "preview" | "pre" => Some(PreReleaseLabel::Rc),
        _ => None,
    }
}

pub fn resolve_timestamp(pattern: &str, timestamp: Option<u64>) -> Result<u64, &'static str> {
    let ts = timestamp.ok_or("Timestamp is required but was None")?;
    let dt = chrono::DateTime::from_timestamp(ts as i64, 0).ok_or("Invalid timestamp")?;

    let result = match pattern {
        "YYYY" => dt.format("%Y").to_string().parse().unwrap_or(0),
        "YY" => dt.format("%y").to_string().parse().unwrap_or(0),
        "MM" => dt.format("%-m").to_string().parse().unwrap_or(0),
        "0M" => dt.format("%m").to_string().parse().unwrap_or(0),
        "WW" => dt.format("%-W").to_string().parse().unwrap_or(0),
        "0W" => dt.format("%W").to_string().parse().unwrap_or(0),
        "DD" => dt.format("%-d").to_string().parse().unwrap_or(0),
        "0D" => dt.format("%d").to_string().parse().unwrap_or(0),
        "HH" => dt.format("%-H").to_string().parse().unwrap_or(0),
        "0H" => dt.format("%H").to_string().parse().unwrap_or(0),
        "mm" => dt.format("%-M").to_string().parse().unwrap_or(0),
        "0m" => dt.format("%M").to_string().parse().unwrap_or(0),
        "SS" => dt.format("%-S").to_string().parse().unwrap_or(0),
        "0S" => dt.format("%S").to_string().parse().unwrap_or(0),
        _ => return Err("Unknown timestamp pattern"),
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
            Err("Timestamp is required but was None")
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
            resolve_timestamp(pattern, Some(timestamp)).unwrap(),
            expected
        );
    }

    #[test]
    fn test_resolve_timestamp_unknown_pattern() {
        let timestamp = Some(1710511845);
        assert_eq!(
            resolve_timestamp("INVALID", timestamp),
            Err("Unknown timestamp pattern")
        );
    }
}
