use std::collections::HashSet;

use crate::error::ZervError;
use crate::version::zerv::core::Zerv;

/// Type alias for process specifications: (index, override_value, bump_value)
pub type ProcessSpec = (usize, Option<String>, Option<String>);

impl Zerv {
    pub fn parse_bump_spec(spec: &str, schema_len: usize) -> Result<(usize, String), ZervError> {
        if let Some((index_str, value)) = spec.split_once('=') {
            // Explicit value: "1=5" -> (1, "5")
            let index = Self::parse_index(index_str, schema_len)?;
            let value = Self::parse_value(value)?;
            Ok((index, value))
        } else {
            // Default value: "1" -> (1, "1")
            let index = Self::parse_index(spec, schema_len)?;
            Ok((index, "1".to_string()))
        }
    }

    fn parse_index(index_str: &str, schema_len: usize) -> Result<usize, ZervError> {
        let index: i32 = index_str
            .parse()
            .map_err(|_| ZervError::InvalidBumpTarget("Invalid index".to_string()))?;

        if index >= 0 {
            // Positive index: 0, 1, 2, ...
            let idx = index as usize;
            if idx >= schema_len {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Index {idx} out of bounds for schema of length {schema_len}"
                )));
            }
            Ok(idx)
        } else {
            // Negative index: -1, -2, -3, ... (count from end)
            let idx = schema_len as i32 + index;
            if idx < 0 || idx >= schema_len as i32 {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Negative index {index} out of bounds for schema of length {schema_len}"
                )));
            }
            Ok(idx as usize)
        }
    }

    fn parse_value(value_str: &str) -> Result<String, ZervError> {
        // For numeric values, ensure they're positive
        if let Ok(num) = value_str.parse::<i32>()
            && num < 0
        {
            return Err(ZervError::InvalidBumpTarget(
                "Negative bump values not supported".to_string(),
            ));
        }

        Ok(value_str.to_string())
    }

    pub fn parse_optional_u32(
        value: Option<&str>,
        field_name: &str,
    ) -> Result<Option<u32>, ZervError> {
        match value {
            Some(val) => {
                let parsed = val.parse::<u32>().map_err(|_| {
                    ZervError::InvalidBumpTarget(format!(
                        "Expected numeric value for {field_name} component, got: {val}"
                    ))
                })?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    pub fn parse_and_validate_process_specs(
        overrides: &[String],
        bumps: &[String],
        schema_len: usize,
    ) -> Result<Vec<ProcessSpec>, ZervError> {
        let mut parsed_specs = Vec::new();
        let mut seen_override_indices = HashSet::new();
        let mut seen_bump_indices = HashSet::new();

        // Parse override specs
        for spec in overrides {
            let (index, value) = Self::parse_override_spec(spec, schema_len)?;

            // Check for duplicate indices within overrides
            if !seen_override_indices.insert(index) {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Duplicate index {index} found in override specifications"
                )));
            }

            parsed_specs.push((index, Some(value), None));
        }

        // Parse bump specs
        for spec in bumps {
            let (index, value) = Self::parse_bump_spec(spec, schema_len)?;

            // Check for duplicate indices within bumps
            if !seen_bump_indices.insert(index) {
                return Err(ZervError::InvalidBumpTarget(format!(
                    "Duplicate index {index} found in bump specifications"
                )));
            }

            // Check if we already have an override for this index
            if let Some(existing) = parsed_specs.iter_mut().find(|(idx, _, _)| *idx == index) {
                // We already have an override for this index, add the bump value
                existing.2 = Some(value);
            } else {
                parsed_specs.push((index, None, Some(value)));
            }
        }

        // Sort specs by index (lower to higher)
        parsed_specs.sort_by_key(|(index, _, _)| *index);

        Ok(parsed_specs)
    }

    pub fn parse_override_spec(
        spec: &str,
        schema_len: usize,
    ) -> Result<(usize, String), ZervError> {
        if let Some((index_str, value)) = spec.split_once('=') {
            // Explicit value: "1=5" -> (1, "5")
            let index = Self::parse_index(index_str, schema_len)?;
            let value = Self::parse_value(value)?;
            Ok((index, value))
        } else {
            // Override specs require explicit values
            Err(ZervError::InvalidBumpTarget(format!(
                "Override specification '{spec}' requires explicit value (use index=value format)"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    // Test parse_bump_spec with various valid inputs
    #[rstest]
    #[case("1=5", 3, 1, "5")] // explicit value
    #[case("1", 3, 1, "1")] // default value
    #[case("-1=3", 3, 2, "3")] // negative index with explicit value
    #[case("-1", 3, 2, "1")] // negative index with default value
    #[case("0=release", 3, 0, "release")] // string value
    fn test_parse_bump_spec_valid(
        #[case] spec: &str,
        #[case] schema_len: usize,
        #[case] expected_index: usize,
        #[case] expected_value: &str,
    ) {
        let (index, value) = Zerv::parse_bump_spec(spec, schema_len).unwrap();
        assert_eq!(index, expected_index);
        assert_eq!(value, expected_value);
    }

    // Test parse_bump_spec with invalid inputs
    #[rstest]
    #[case("5", 3)] // index out of bounds
    #[case("-5", 3)] // negative index out of bounds
    #[case("0=-1", 3)] // negative value
    fn test_parse_bump_spec_invalid(#[case] spec: &str, #[case] schema_len: usize) {
        let result = Zerv::parse_bump_spec(spec, schema_len);
        assert!(result.is_err());
    }

    // Test parse_and_validate_process_specs with valid inputs
    #[rstest]
    #[case(&[], &[], vec![])] // empty
    #[case(&["1=5"], &[], vec![(1, Some("5".to_string()), None)])] // single override
    #[case(
        &["2=3", "0=1"],
        &["1=2"],
        vec![
            (0, Some("1".to_string()), None), // Override
            (1, None, Some("2".to_string())), // Bump
            (2, Some("3".to_string()), None), // Override
        ]
    )] // multiple sorted
    #[case(
        &["-1=3", "0=1"],
        &["-2=2"],
        vec![
            (0, Some("1".to_string()), None), // Override
            (1, None, Some("2".to_string())), // Bump (-2 maps to 1)
            (2, Some("3".to_string()), None), // Override (-1 maps to 2)
        ]
    )] // negative indices
    #[case(
        &["0=5"],
        &["1=3", "2=1"],
        vec![
            (0, Some("5".to_string()), None), // Override
            (1, None, Some("3".to_string())), // Bump
            (2, None, Some("1".to_string())), // Bump
        ]
    )] // mixed override and bump
    #[case(
        &["1=release"],
        &["0=alpha"],
        vec![
            (0, None, Some("alpha".to_string())),   // Bump
            (1, Some("release".to_string()), None), // Override
        ]
    )] // string values
    #[case(
        &["1=10", "2=5"],
        &["1=3", "0=2"],
        vec![
            (0, None, Some("2".to_string())),       // Bump only
            (1, Some("10".to_string()), Some("3".to_string())), // Both override and bump
            (2, Some("5".to_string()), None),       // Override only
        ]
    )] // mixed: some indices have both override and bump
    fn test_parse_and_validate_process_specs_valid(
        #[case] overrides: &[&str],
        #[case] bumps: &[&str],
        #[case] expected: Vec<(usize, Option<String>, Option<String>)>,
    ) {
        let override_strings: Vec<String> = overrides.iter().map(|s| s.to_string()).collect();
        let bump_strings: Vec<String> = bumps.iter().map(|s| s.to_string()).collect();

        let specs =
            Zerv::parse_and_validate_process_specs(&override_strings, &bump_strings, 3).unwrap();
        assert_eq!(specs, expected);
    }

    // Test parse_and_validate_process_specs with invalid inputs
    #[rstest]
    #[case(
        &["1=5", "1=1"],
        &["0=3"],
        "Duplicate index 1"
    )] // duplicate indices in overrides
    #[case(
        &["-1=5", "0=1"],
        &["-1=3", "-1=2"],
        "Duplicate index 2"
    )] // duplicate negative indices in bumps (-1 maps to 2)
    #[case(
        &["1=5"],
        &["5=1"],
        "out of bounds"
    )] // index out of bounds
    #[case(
        &["1=5"],
        &["-5=1"],
        "out of bounds"
    )] // negative index out of bounds
    #[case(
        &[],
        &["1=-5"],
        "Negative bump values not supported"
    )] // negative values
    fn test_parse_and_validate_process_specs_invalid(
        #[case] overrides: &[&str],
        #[case] bumps: &[&str],
        #[case] expected_error: &str,
    ) {
        let override_strings: Vec<String> = overrides.iter().map(|s| s.to_string()).collect();
        let bump_strings: Vec<String> = bumps.iter().map(|s| s.to_string()).collect();

        let result = Zerv::parse_and_validate_process_specs(&override_strings, &bump_strings, 3);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    // Test parse_override_spec with valid inputs
    #[rstest]
    #[case("1=5", 3, 1, "5")] // explicit value
    #[case("-1=3", 3, 2, "3")] // negative index (-1 maps to last index 2)
    #[case("0=release", 3, 0, "release")] // string value
    fn test_parse_override_spec_valid(
        #[case] spec: &str,
        #[case] schema_len: usize,
        #[case] expected_index: usize,
        #[case] expected_value: &str,
    ) {
        let (index, value) = Zerv::parse_override_spec(spec, schema_len).unwrap();
        assert_eq!(index, expected_index);
        assert_eq!(value, expected_value);
    }

    // Test parse_override_spec with invalid inputs
    #[rstest]
    #[case("1", 3, "requires explicit value")] // missing value
    #[case("5=1", 3, "out of bounds")] // invalid index
    fn test_parse_override_spec_invalid(
        #[case] spec: &str,
        #[case] schema_len: usize,
        #[case] expected_error: &str,
    ) {
        let result = Zerv::parse_override_spec(spec, schema_len);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }
}
