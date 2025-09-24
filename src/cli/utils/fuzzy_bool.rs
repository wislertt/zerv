use std::fmt;
use std::str::FromStr;

/// A flexible boolean type that accepts various string representations
/// Supports: true/false, t/f, yes/no, y/n, 1/0, on/off (case-insensitive)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyBool(pub bool);

impl FuzzyBool {
    /// Create a new FuzzyBool with the given boolean value
    pub fn new(value: bool) -> Self {
        FuzzyBool(value)
    }

    /// Get the inner boolean value
    pub fn value(&self) -> bool {
        self.0
    }
}

impl FromStr for FuzzyBool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // True values
            "true" | "t" | "yes" | "y" | "1" | "on" => Ok(FuzzyBool(true)),
            // False values
            "false" | "f" | "no" | "n" | "0" | "off" => Ok(FuzzyBool(false)),
            // Invalid value
            _ => Err(format!(
                "Invalid boolean value: '{s}'. Supported values: true/false, t/f, yes/no, y/n, 1/0, on/off (case-insensitive)"
            )),
        }
    }
}

impl fmt::Display for FuzzyBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<bool> for FuzzyBool {
    fn from(value: bool) -> Self {
        FuzzyBool(value)
    }
}

impl From<FuzzyBool> for bool {
    fn from(fuzzy: FuzzyBool) -> Self {
        fuzzy.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_values() {
        let true_values = ["true", "t", "yes", "y", "1", "on"];

        for value in &true_values {
            let result = FuzzyBool::from_str(value);
            assert!(result.is_ok(), "Failed to parse '{value}' as true");
            assert!(result.unwrap().value(), "Value '{value}' should be true");
        }
    }

    #[test]
    fn test_false_values() {
        let false_values = ["false", "f", "no", "n", "0", "off"];

        for value in &false_values {
            let result = FuzzyBool::from_str(value);
            assert!(result.is_ok(), "Failed to parse '{value}' as false");
            assert!(!result.unwrap().value(), "Value '{value}' should be false");
        }
    }

    #[test]
    fn test_case_insensitive_true() {
        let case_variations = [
            "TRUE", "True", "tRuE", "T", "YES", "Yes", "yEs", "Y", "ON", "On", "oN",
        ];

        for value in &case_variations {
            let result = FuzzyBool::from_str(value);
            assert!(
                result.is_ok(),
                "Failed to parse '{value}' as true (case insensitive)"
            );
            assert!(result.unwrap().value(), "Value '{value}' should be true");
        }
    }

    #[test]
    fn test_case_insensitive_false() {
        let case_variations = [
            "FALSE", "False", "fAlSe", "F", "NO", "No", "nO", "N", "OFF", "Off", "oFf",
        ];

        for value in &case_variations {
            let result = FuzzyBool::from_str(value);
            assert!(
                result.is_ok(),
                "Failed to parse '{value}' as false (case insensitive)"
            );
            assert!(!result.unwrap().value(), "Value '{value}' should be false");
        }
    }

    #[test]
    fn test_invalid_values() {
        let invalid_values = [
            "maybe", "unknown", "2", "-1", "true1", "false0", "yep", "nope", "enable", "disable",
            "", " ", "null",
        ];

        for value in &invalid_values {
            let result = FuzzyBool::from_str(value);
            assert!(
                result.is_err(),
                "Should fail to parse invalid value '{value}'"
            );

            let error_msg = result.unwrap_err();
            assert!(
                error_msg.contains("Invalid boolean value"),
                "Error message should mention invalid boolean value for '{value}'"
            );
            assert!(
                error_msg.contains("true/false, t/f, yes/no, y/n, 1/0, on/off"),
                "Error message should list supported values for '{value}'"
            );
        }
    }

    #[test]
    fn test_new_constructor() {
        let fuzzy_true = FuzzyBool::new(true);
        assert!(fuzzy_true.value());

        let fuzzy_false = FuzzyBool::new(false);
        assert!(!fuzzy_false.value());
    }

    #[test]
    fn test_display_trait() {
        let fuzzy_true = FuzzyBool::new(true);
        assert_eq!(format!("{fuzzy_true}"), "true");

        let fuzzy_false = FuzzyBool::new(false);
        assert_eq!(format!("{fuzzy_false}"), "false");
    }

    #[test]
    fn test_from_bool() {
        let fuzzy_true: FuzzyBool = true.into();
        assert!(fuzzy_true.value());

        let fuzzy_false: FuzzyBool = false.into();
        assert!(!fuzzy_false.value());
    }

    #[test]
    fn test_into_bool() {
        let fuzzy_true = FuzzyBool::new(true);
        let bool_value: bool = fuzzy_true.into();
        assert!(bool_value);

        let fuzzy_false = FuzzyBool::new(false);
        let bool_value: bool = fuzzy_false.into();
        assert!(!bool_value);
    }

    #[test]
    fn test_equality() {
        let fuzzy_true1 = FuzzyBool::new(true);
        let fuzzy_true2 = FuzzyBool::from_str("yes").unwrap();
        assert_eq!(fuzzy_true1, fuzzy_true2);

        let fuzzy_false1 = FuzzyBool::new(false);
        let fuzzy_false2 = FuzzyBool::from_str("no").unwrap();
        assert_eq!(fuzzy_false1, fuzzy_false2);

        assert_ne!(fuzzy_true1, fuzzy_false1);
    }

    #[test]
    fn test_clone_and_copy() {
        let original = FuzzyBool::new(true);
        let cloned = original; // Use copy instead of clone since FuzzyBool implements Copy
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);
    }

    #[test]
    fn test_debug_trait() {
        let fuzzy_true = FuzzyBool::new(true);
        let debug_str = format!("{fuzzy_true:?}");
        assert!(debug_str.contains("FuzzyBool"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_comprehensive_parsing() {
        // Test all supported true values with various cases
        let all_true_values = [
            "true", "TRUE", "True", "tRuE", "t", "T", "yes", "YES", "Yes", "yEs", "y", "Y", "1",
            "on", "ON", "On", "oN",
        ];

        for value in &all_true_values {
            let parsed = FuzzyBool::from_str(value)
                .unwrap_or_else(|e| panic!("Failed to parse '{value}': {e}"));
            assert!(parsed.value(), "Value '{value}' should parse to true");
        }

        // Test all supported false values with various cases
        let all_false_values = [
            "false", "FALSE", "False", "fAlSe", "f", "F", "no", "NO", "No", "nO", "n", "N", "0",
            "off", "OFF", "Off", "oFf",
        ];

        for value in &all_false_values {
            let parsed = FuzzyBool::from_str(value)
                .unwrap_or_else(|e| panic!("Failed to parse '{value}': {e}"));
            assert!(!parsed.value(), "Value '{value}' should parse to false");
        }
    }
}
