use crate::constants::{ron_fields, timestamp_patterns};
use crate::error::ZervError;
use crate::version::zerv::Component;

/// Component validation utilities
pub mod component_validation {
    use super::*;

    /// Get all valid field names for var() components
    pub fn get_valid_field_names() -> Vec<&'static str> {
        vec![
            // Core version fields
            ron_fields::MAJOR,
            ron_fields::MINOR,
            ron_fields::PATCH,
            ron_fields::EPOCH,
            // Pre-release fields
            ron_fields::PRE_RELEASE,
            // Post-release fields
            ron_fields::POST,
            ron_fields::DEV,
            // VCS state fields
            ron_fields::DISTANCE,
            ron_fields::DIRTY,
            ron_fields::BRANCH,
            ron_fields::COMMIT_HASH_SHORT,
            // Last version fields
            ron_fields::LAST_BRANCH,
            ron_fields::LAST_COMMIT_HASH,
            ron_fields::LAST_TIMESTAMP,
        ]
    }

    /// Check if a field name is valid for var() components
    pub fn is_valid_var_field_name(field_name: &str) -> bool {
        // Check exact matches first
        if get_valid_field_names().contains(&field_name) {
            return true;
        }

        // Check for custom fields (custom.*) - must have something after the dot
        if field_name.starts_with(&format!("{}.", ron_fields::CUSTOM))
            && field_name.len() > ron_fields::CUSTOM.len() + 1
        {
            return true;
        }

        false
    }

    /// Get all valid timestamp patterns for ts() components
    pub fn get_valid_timestamp_patterns() -> Vec<&'static str> {
        timestamp_patterns::get_valid_timestamp_patterns()
    }

    /// Check if a timestamp pattern is valid for ts() components
    pub fn is_valid_timestamp_pattern(pattern: &str) -> bool {
        // Check preset patterns
        if get_valid_timestamp_patterns().contains(&pattern) {
            return true;
        }

        // Check for custom chrono format strings (start with %)
        if pattern.starts_with('%') {
            return true;
        }

        false
    }

    /// Validate a single component
    pub fn validate_component(component: &Component) -> Result<(), ZervError> {
        match component {
            Component::VarField(field_name) => {
                if !is_valid_var_field_name(field_name) {
                    let valid_fields = get_valid_field_names();
                    return Err(ZervError::StdinError(format!(
                        "Invalid Zerv RON: unknown field '{}' in var() component. Valid fields are: {}",
                        field_name,
                        valid_fields.join(", ")
                    )));
                }
            }
            Component::VarTimestamp(pattern) => {
                if !is_valid_timestamp_pattern(pattern) {
                    let valid_patterns = get_valid_timestamp_patterns();
                    return Err(ZervError::StdinError(format!(
                        "Invalid Zerv RON: unknown timestamp pattern '{}' in ts() component. Valid patterns are: {} or custom format starting with %",
                        pattern,
                        valid_patterns.join(", ")
                    )));
                }
            }
            Component::String(_) => {
                // String components are always valid
            }
            Component::Integer(_) => {
                // Integer components are always valid
            }
        }
        Ok(())
    }

    /// Validate all components in a schema
    pub fn validate_schema_components(components: &[Component]) -> Result<(), ZervError> {
        for component in components {
            validate_component(component)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("major")]
    #[case("minor")]
    #[case("patch")]
    #[case("epoch")]
    #[case("pre_release")]
    #[case("post")]
    #[case("dev")]
    #[case("distance")]
    #[case("dirty")]
    #[case("branch")]
    #[case("commit_hash_short")]
    #[case("last_branch")]
    #[case("last_commit_hash")]
    #[case("last_timestamp")]
    #[case("custom.build_id")]
    #[case("custom.environment")]
    #[case("custom.metadata.author")]
    fn test_validate_component_valid_var_field(#[case] field_name: &str) {
        let component = Component::VarField(field_name.to_string());
        assert!(component_validation::validate_component(&component).is_ok());
    }

    #[rstest]
    #[case("invalid_field")]
    #[case("unknown")]
    #[case("bad_field")]
    #[case("invalid.custom")]
    #[case("custom.")]
    #[case("")]
    #[case("custom")]
    fn test_validate_component_invalid_var_field(#[case] field_name: &str) {
        let component = Component::VarField(field_name.to_string());
        let result = component_validation::validate_component(&component);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[rstest]
    #[case("compact_date")]
    #[case("compact_datetime")]
    #[case("YYYY")]
    #[case("YY")]
    #[case("MM")]
    #[case("0M")]
    #[case("DD")]
    #[case("0D")]
    #[case("HH")]
    #[case("0H")]
    #[case("mm")]
    #[case("0m")]
    #[case("SS")]
    #[case("0S")]
    #[case("WW")]
    #[case("0W")]
    #[case("%Y-%m-%d")]
    #[case("%Y%m%d")]
    #[case("%H:%M:%S")]
    #[case("%Y%m%d%H%M%S")]
    fn test_validate_component_valid_timestamp(#[case] pattern: &str) {
        let component = Component::VarTimestamp(pattern.to_string());
        assert!(component_validation::validate_component(&component).is_ok());
    }

    #[rstest]
    #[case("INVALID")]
    #[case("bad_pattern")]
    #[case("YYYYMMDD")]
    #[case("HHmmss")]
    #[case("invalid_format")]
    #[case("")]
    fn test_validate_component_invalid_timestamp(#[case] pattern: &str) {
        let component = Component::VarTimestamp(pattern.to_string());
        let result = component_validation::validate_component(&component);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unknown timestamp pattern")
        );
    }

    #[rstest]
    #[case("test")]
    #[case("hello world")]
    #[case("")]
    #[case("special-chars!@#$%")]
    #[case("unicode_测试")]
    fn test_validate_component_string(#[case] value: &str) {
        let component = Component::String(value.to_string());
        assert!(component_validation::validate_component(&component).is_ok());
    }

    #[rstest]
    #[case(0)]
    #[case(42)]
    #[case(1234567890)]
    #[case(u64::MAX)]
    fn test_validate_component_integer(#[case] value: u64) {
        let component = Component::Integer(value);
        assert!(component_validation::validate_component(&component).is_ok());
    }
}
