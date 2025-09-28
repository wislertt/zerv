use crate::constants::timestamp_patterns;
use crate::error::{Result, ZervError};

fn tokenize_pattern(pattern: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut previous_c = None;

    for c in pattern.chars() {
        if c == '0' {
            // Start a new token with '0'
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
            current_token.push(c);
        } else if previous_c == Some('0') || (previous_c == Some(c) && is_pattern_char(c)) {
            // Continue current token if previous was '0' or same pattern character
            current_token.push(c);
        } else if is_pattern_char(c) {
            // Start a new pattern token
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
            current_token.push(c);
        } else {
            // Reject any non-pattern characters
            return Err(ZervError::InvalidFormat(format!(
                "Invalid character '{c}' in pattern '{pattern}'. Only pattern characters (Y, M, D, H, m, S, W, 0) are supported. Use chrono format strings for literals."
            )));
        }
        previous_c = Some(c);
    }

    // Process the last token
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    // Validate that all tokens are valid timestamp patterns
    let valid_patterns = timestamp_patterns::get_valid_timestamp_patterns();
    for token in &tokens {
        if !valid_patterns.contains(&token.as_str()) {
            return Err(ZervError::InvalidFormat(format!(
                "Invalid timestamp pattern '{token}'. Use valid timestamp patterns like YYYY, YY, MM, etc."
            )));
        }
    }

    Ok(tokens)
}

fn is_pattern_char(c: char) -> bool {
    matches!(c, 'Y' | 'M' | 'D' | 'H' | 'm' | 'S' | 'W')
}

fn parse_timestamp_component(dt: &chrono::DateTime<chrono::Utc>, format_str: &str) -> String {
    dt.format(format_str).to_string()
}

pub fn resolve_timestamp(pattern: &str, timestamp: u64) -> Result<String> {
    let dt = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .ok_or_else(|| ZervError::InvalidFormat("Invalid timestamp".to_string()))?;

    // Handle compact patterns directly without tokenization
    match pattern {
        timestamp_patterns::COMPACT_DATE => {
            // YYYY0M0D format (e.g., 20240315)
            return Ok(parse_timestamp_component(&dt, "%Y%m%d"));
        }
        timestamp_patterns::COMPACT_DATETIME => {
            // YYYY0M0D0H0m0S format (e.g., 20240315141045)
            return Ok(parse_timestamp_component(&dt, "%Y%m%d%H%M%S"));
        }
        _ => {
            // Continue with tokenization for other patterns
        }
    }

    let tokens = tokenize_pattern(pattern)?;
    let mut result = Vec::new();

    for token in tokens {
        let resolved_token = match token.as_str() {
            timestamp_patterns::YYYY => parse_timestamp_component(&dt, "%Y"),
            timestamp_patterns::YY => parse_timestamp_component(&dt, "%y"),
            timestamp_patterns::MM => parse_timestamp_component(&dt, "%-m"),
            timestamp_patterns::ZERO_M => parse_timestamp_component(&dt, "%m"),
            timestamp_patterns::WW => parse_timestamp_component(&dt, "%-W"),
            timestamp_patterns::ZERO_W => parse_timestamp_component(&dt, "%W"),
            timestamp_patterns::DD => parse_timestamp_component(&dt, "%-d"),
            timestamp_patterns::ZERO_D => parse_timestamp_component(&dt, "%d"),
            timestamp_patterns::HH => parse_timestamp_component(&dt, "%-H"),
            timestamp_patterns::ZERO_H => parse_timestamp_component(&dt, "%H"),
            timestamp_patterns::MM_MINUTE => parse_timestamp_component(&dt, "%-M"),
            timestamp_patterns::ZERO_M_MINUTE => parse_timestamp_component(&dt, "%M"),
            timestamp_patterns::SS => parse_timestamp_component(&dt, "%-S"),
            timestamp_patterns::ZERO_S => parse_timestamp_component(&dt, "%S"),
            _ => token.clone(), // Treat as literal
        };
        result.push(resolved_token);
    }

    let result_string = result.join("");
    Ok(result_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1710511845, timestamp_patterns::YYYY, "2024")] // 2024-03-15 14:10:45
    #[case(1710511845, timestamp_patterns::YY, "24")]
    #[case(1710511845, timestamp_patterns::MM, "3")]
    #[case(1710511845, timestamp_patterns::ZERO_M, "03")]
    #[case(1710511845, timestamp_patterns::WW, "11")]
    #[case(1710511845, timestamp_patterns::ZERO_W, "11")]
    #[case(1710511845, timestamp_patterns::DD, "15")]
    #[case(1710511845, timestamp_patterns::ZERO_D, "15")]
    #[case(1710511845, timestamp_patterns::HH, "14")]
    #[case(1710511845, timestamp_patterns::ZERO_H, "14")]
    #[case(1710511845, timestamp_patterns::MM_MINUTE, "10")]
    #[case(1710511845, timestamp_patterns::ZERO_M_MINUTE, "10")]
    #[case(1710511845, timestamp_patterns::SS, "45")]
    #[case(1710511845, timestamp_patterns::ZERO_S, "45")]
    #[case(1577836800, timestamp_patterns::YYYY, "2020")] // 2020-01-01 00:00:00 - test leading zeros
    #[case(1577836800, timestamp_patterns::MM, "1")]
    #[case(1577836800, timestamp_patterns::ZERO_M, "01")]
    #[case(1577836800, timestamp_patterns::DD, "1")]
    #[case(1577836800, timestamp_patterns::ZERO_D, "01")]
    #[case(1577836800, timestamp_patterns::WW, "0")]
    #[case(1577836800, timestamp_patterns::ZERO_W, "00")]
    #[case(1577836800, timestamp_patterns::HH, "0")]
    #[case(1577836800, timestamp_patterns::ZERO_H, "00")]
    #[case(1577836800, timestamp_patterns::MM_MINUTE, "0")]
    #[case(1577836800, timestamp_patterns::ZERO_M_MINUTE, "00")]
    #[case(1577836800, timestamp_patterns::SS, "0")]
    #[case(1577836800, timestamp_patterns::ZERO_S, "00")]
    #[case(1609459200, timestamp_patterns::MM, "1")] // 2021-01-01 00:00:00 - different year
    #[case(1609459200, timestamp_patterns::ZERO_M, "01")]
    #[case(1609459200, timestamp_patterns::WW, "0")]
    #[case(1609459200, timestamp_patterns::ZERO_W, "00")]
    // Compact pattern tests
    #[case(1710511845, timestamp_patterns::COMPACT_DATE, "20240315")] // 2024-03-15 14:10:45
    #[case(1710511845, timestamp_patterns::COMPACT_DATETIME, "20240315141045")]
    #[case(1577836800, timestamp_patterns::COMPACT_DATE, "20200101")] // 2020-01-01 00:00:00
    #[case(1577836800, timestamp_patterns::COMPACT_DATETIME, "20200101000000")]
    #[case(1609459200, timestamp_patterns::COMPACT_DATE, "20210101")] // 2021-01-01 00:00:00
    #[case(1609459200, timestamp_patterns::COMPACT_DATETIME, "20210101000000")]
    fn test_resolve_timestamp_patterns(
        #[case] timestamp: u64,
        #[case] pattern: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(resolve_timestamp(pattern, timestamp).unwrap(), expected);
    }

    #[test]
    fn test_resolve_timestamp_unknown_pattern() {
        let timestamp = 1710511845;
        // "INVALID" will be treated as a literal, so it should fail to parse as u64
        assert!(resolve_timestamp("INVALID", timestamp).is_err());
    }

    // Tests for new tokenization functionality - only pattern characters
    #[rstest]
    #[case("YYYY0M", vec!["YYYY", "0M"])]
    #[case("YYMMDD", vec!["YY", "MM", "DD"])]
    #[case("YYYY0M0D", vec!["YYYY", "0M", "0D"])]
    #[case("YYYY0M0DHHmmSS", vec!["YYYY", "0M", "0D", "HH", "mm", "SS"])]
    fn test_tokenize_patterns(#[case] pattern: &str, #[case] expected: Vec<&str>) {
        let tokens = tokenize_pattern(pattern).unwrap();
        assert_eq!(tokens, expected);
    }

    #[rstest]
    #[case("YYYY-0M")]
    #[case("YYYY_0M")]
    #[case("YYYY.0M")]
    #[case("YYYY 0M")]
    #[case("YYYY|0M")]
    #[case("YYYY#0M")]
    #[case("Y")]
    #[case("YYY")]
    #[case("M")]
    #[case("D")]
    #[case("H")]
    #[case("S")]
    #[case("W")]
    #[case("m")]
    #[case("YYM")]
    #[case("MMD")]
    #[case("DDH")]
    #[case("HHS")]
    #[case("SSW")]
    #[case("WWm")]
    fn test_tokenize_pattern_invalid(#[case] pattern: &str) {
        assert!(tokenize_pattern(pattern).is_err());
    }

    #[test]
    fn test_debug_tokenization() {
        // Debug test to see what tokens are generated
        let tokens = tokenize_pattern("YYYY0M").unwrap();
        println!("YYYY0M tokens: {tokens:?}");

        let tokens = tokenize_pattern("YYMMDD").unwrap();
        println!("YYMMDD tokens: {tokens:?}");

        let tokens = tokenize_pattern("YYYY0M0D").unwrap();
        println!("YYYY0M0D tokens: {tokens:?}");
    }

    // Tests for resolve_timestamp with new patterns - only pattern characters
    #[rstest]
    #[case(1710511845, "YYYY0M", "202403")] // 2024-03-15 14:10:45
    #[case(1710511845, "YYMMDD", "24315")]
    #[case(1710511845, "YYYY0M0D", "20240315")]
    #[case(1710511845, "YYYY0M0DHHmmSS", "20240315141045")]
    #[case(1577836800, "YYYY0M0D", "20200101")] // 2020-01-01 00:00:00
    #[case(1577836800, "YY0M0D", "200101")]
    #[case(1609459200, "YYYY0M0D", "20210101")] // 2021-01-01 00:00:00
    fn test_resolve_timestamp_combined_patterns(
        #[case] timestamp: u64,
        #[case] pattern: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(resolve_timestamp(pattern, timestamp).unwrap(), expected);
    }

    #[test]
    fn test_resolve_timestamp_invalid_combined_pattern() {
        let timestamp = 1710511845;
        assert!(resolve_timestamp("Y", timestamp).is_err());
        assert!(resolve_timestamp("M", timestamp).is_err());
        assert!(resolve_timestamp("D", timestamp).is_err());
        assert!(resolve_timestamp("H", timestamp).is_err());
        assert!(resolve_timestamp("S", timestamp).is_err());
        assert!(resolve_timestamp("W", timestamp).is_err());
        assert!(resolve_timestamp("m", timestamp).is_err());
        // These should fail due to invalid characters (dash and underscore)
        assert!(resolve_timestamp("YYYY-0M", timestamp).is_err()); // literal dash
        assert!(resolve_timestamp("YYYY_0M", timestamp).is_err()); // literal underscore
    }
}
