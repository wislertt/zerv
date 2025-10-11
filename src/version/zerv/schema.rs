use serde::{
    Deserialize,
    Serialize,
};

use super::PrecedenceOrder;
use super::components::{
    Component,
    Var,
};
use crate::constants::{
    ron_fields,
    timestamp_patterns,
};
use crate::error::ZervError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZervSchema {
    pub core: Vec<Component>,
    pub extra_core: Vec<Component>,
    pub build: Vec<Component>,
    #[serde(default)]
    pub precedence_order: PrecedenceOrder,
}

impl ZervSchema {
    /// Create a new ZervSchema with automatic validation and default precedence order
    pub fn new(
        core: Vec<Component>,
        extra_core: Vec<Component>,
        build: Vec<Component>,
    ) -> Result<Self, ZervError> {
        Self::new_with_precedence(core, extra_core, build, PrecedenceOrder::default())
    }

    /// Create a new ZervSchema with custom precedence order
    pub fn new_with_precedence(
        core: Vec<Component>,
        extra_core: Vec<Component>,
        build: Vec<Component>,
        precedence_order: PrecedenceOrder,
    ) -> Result<Self, ZervError> {
        let schema = Self {
            core,
            extra_core,
            build,
            precedence_order,
        };
        schema.validate()?;
        Ok(schema)
    }

    /// Validate the schema structure and components
    pub fn validate(&self) -> Result<(), ZervError> {
        // Check that schema has at least one component
        if self.core.is_empty() && self.extra_core.is_empty() && self.build.is_empty() {
            return Err(ZervError::StdinError(
                "Invalid Zerv RON: schema must contain at least one component in core, extra_core, or build sections".to_string()
            ));
        }

        // Validate all schema components
        Self::validate_components(&self.core)?;
        Self::validate_components(&self.extra_core)?;
        Self::validate_components(&self.build)?;

        Ok(())
    }

    /// Get the PEP440-based precedence order
    pub fn pep440_based_precedence_order() -> PrecedenceOrder {
        PrecedenceOrder::pep440_based()
    }

    /// Validate a single component
    fn validate_component(component: &Component) -> Result<(), ZervError> {
        match component {
            Component::Var(var) => {
                match var {
                    Var::Custom(field_name) => {
                        if !Self::is_valid_var_field_name(field_name) {
                            let valid_fields = Self::get_valid_field_names();
                            return Err(ZervError::StdinError(format!(
                                "Invalid Zerv RON: unknown field '{}' in var() component. Valid fields are: {}",
                                field_name,
                                valid_fields.join(", ")
                            )));
                        }
                    }
                    Var::Timestamp(pattern) => {
                        if !Self::is_valid_timestamp_pattern(pattern) {
                            let valid_patterns = Self::get_valid_timestamp_patterns();
                            return Err(ZervError::StdinError(format!(
                                "Invalid Zerv RON: unknown timestamp pattern '{}' in ts() component. Valid patterns are: {} or custom format starting with %",
                                pattern,
                                valid_patterns.join(", ")
                            )));
                        }
                    }
                    _ => {} // Standard enum variants are always valid
                }
            }
            Component::Str(_) => {}
            Component::Int(_) => {}
        }
        Ok(())
    }

    /// Validate all components in a schema
    fn validate_components(components: &[Component]) -> Result<(), ZervError> {
        for component in components {
            Self::validate_component(component)?;
        }
        Ok(())
    }

    /// Get all valid field names for var() components
    fn get_valid_field_names() -> Vec<&'static str> {
        vec![
            // Core version fields
            ron_fields::MAJOR,
            ron_fields::MINOR,
            ron_fields::PATCH,
            ron_fields::EPOCH,
            // Pre-release fields
            ron_fields::PRE_RELEASE,
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
    fn is_valid_var_field_name(field_name: &str) -> bool {
        // Check exact matches first
        if Self::get_valid_field_names().contains(&field_name) {
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
    fn get_valid_timestamp_patterns() -> Vec<&'static str> {
        timestamp_patterns::get_valid_timestamp_patterns()
    }

    /// Check if a timestamp pattern is valid for ts() components
    fn is_valid_timestamp_pattern(pattern: &str) -> bool {
        // Check preset patterns
        if Self::get_valid_timestamp_patterns().contains(&pattern) {
            return true;
        }

        // Check for custom chrono format strings (start with %)
        if pattern.starts_with('%') {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::zerv::bump::precedence::{
        Precedence,
        PrecedenceOrder,
    };

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
        let component = Component::Var(Var::Custom(field_name.to_string()));
        assert!(ZervSchema::validate_component(&component).is_ok());
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
        let component = Component::Var(Var::Custom(field_name.to_string()));
        let result = ZervSchema::validate_component(&component);
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
        let component = Component::Var(Var::Timestamp(pattern.to_string()));
        assert!(ZervSchema::validate_component(&component).is_ok());
    }

    #[rstest]
    #[case("INVALID")]
    #[case("bad_pattern")]
    #[case("YYYYMMDD")]
    #[case("HHmmss")]
    #[case("invalid_format")]
    #[case("")]
    fn test_validate_component_invalid_timestamp(#[case] pattern: &str) {
        let component = Component::Var(Var::Timestamp(pattern.to_string()));
        let result = ZervSchema::validate_component(&component);
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
        let component = Component::Str(value.to_string());
        assert!(ZervSchema::validate_component(&component).is_ok());
    }

    #[rstest]
    #[case(0)]
    #[case(42)]
    #[case(1234567890)]
    #[case(u64::MAX)]
    fn test_validate_component_integer(#[case] value: u64) {
        let component = Component::Int(value);
        assert!(ZervSchema::validate_component(&component).is_ok());
    }

    #[test]
    fn test_validate_schema_structure_valid() {
        let schema = ZervSchema {
            core: vec![Component::Var(Var::Major)],
            extra_core: vec![],
            build: vec![],
            precedence_order: PrecedenceOrder::default(),
        };
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_validate_schema_structure_empty() {
        let schema = ZervSchema {
            core: vec![],
            extra_core: vec![],
            build: vec![],
            precedence_order: PrecedenceOrder::default(),
        };
        let result = schema.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("schema must contain at least one component")
        );
    }

    #[test]
    fn test_zerv_schema_new_with_validation() {
        let schema = ZervSchema::new(vec![Component::Var(Var::Major)], vec![], vec![]).unwrap();
        assert_eq!(schema.core.len(), 1);
    }

    #[test]
    fn test_zerv_schema_new_invalid() {
        let result = ZervSchema::new(vec![], vec![], vec![]);
        assert!(result.is_err());
    }

    #[rstest]
    #[case(
        vec![
            Component::Var(Var::Major),
            Component::Str(".".to_string()),
            Component::Var(Var::Minor),
        ],
        vec![],
        vec![],
        3, 0, 0
    )]
    #[case(
        vec![],
        vec![
            Component::Var(Var::PreRelease),
            Component::Str("-".to_string()),
            Component::Int(42),
        ],
        vec![],
        0, 3, 0
    )]
    #[case(
        vec![],
        vec![],
        vec![
            Component::Var(Var::Distance),
            Component::Str("+".to_string()),
            Component::Var(Var::Timestamp("compact_date".to_string())),
        ],
        0, 0, 3
    )]
    #[case(
        vec![Component::Var(Var::Major)],
        vec![Component::Var(Var::Minor)],
        vec![Component::Var(Var::Patch)],
        1, 1, 1
    )]
    fn test_zerv_schema_new_valid_sections(
        #[case] core: Vec<Component>,
        #[case] extra_core: Vec<Component>,
        #[case] build: Vec<Component>,
        #[case] expected_core_len: usize,
        #[case] expected_extra_core_len: usize,
        #[case] expected_build_len: usize,
    ) {
        let schema = ZervSchema::new(core, extra_core, build).unwrap();
        assert_eq!(schema.core.len(), expected_core_len);
        assert_eq!(schema.extra_core.len(), expected_extra_core_len);
        assert_eq!(schema.build.len(), expected_build_len);
    }

    #[test]
    fn test_zerv_schema_new_valid_all_component_types() {
        let schema = ZervSchema::new(
            vec![
                Component::Var(Var::Major),
                Component::Str(".".to_string()),
                Component::Int(1),
                Component::Var(Var::Timestamp("YYYY".to_string())),
            ],
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(schema.core.len(), 4);
    }

    #[test]
    fn test_zerv_schema_with_precedence_order() {
        let custom_precedence = PrecedenceOrder::from_precedences(vec![
            Precedence::Major,
            Precedence::Minor,
            Precedence::Patch,
        ]);

        let schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major)],
            vec![],
            vec![],
            custom_precedence.clone(),
        )
        .unwrap();

        assert_eq!(schema.precedence_order.len(), 3);
        assert_eq!(
            schema.precedence_order.get_precedence(0),
            Some(&Precedence::Major)
        );
        assert_eq!(
            schema.precedence_order.get_precedence(1),
            Some(&Precedence::Minor)
        );
        assert_eq!(
            schema.precedence_order.get_precedence(2),
            Some(&Precedence::Patch)
        );
    }

    #[test]
    fn test_zerv_schema_default_precedence_order() {
        let schema = ZervSchema::new(vec![Component::Var(Var::Major)], vec![], vec![]).unwrap();

        // Should use default PEP440-based precedence order
        assert_eq!(schema.precedence_order.len(), 11);
        assert_eq!(
            schema.precedence_order.get_precedence(0),
            Some(&Precedence::Epoch)
        );
        assert_eq!(
            schema.precedence_order.get_precedence(1),
            Some(&Precedence::Major)
        );
        assert_eq!(
            schema.precedence_order.get_precedence(10),
            Some(&Precedence::Build)
        );
    }

    #[test]
    fn test_zerv_schema_pep440_based_precedence_order() {
        let precedence_order = ZervSchema::pep440_based_precedence_order();
        assert_eq!(precedence_order.len(), 11);
        assert_eq!(precedence_order.get_precedence(0), Some(&Precedence::Epoch));
        assert_eq!(precedence_order.get_precedence(1), Some(&Precedence::Major));
        assert_eq!(
            precedence_order.get_precedence(10),
            Some(&Precedence::Build)
        );
    }

    #[rstest]
    #[case("custom.build_id")]
    #[case("custom.environment")]
    #[case("custom.metadata.author")]
    #[case("custom.build.config.debug")]
    fn test_zerv_schema_new_valid_custom_fields(#[case] field_name: &str) {
        let schema = ZervSchema::new(
            vec![
                Component::Var(Var::Major),
                Component::Var(Var::Custom(field_name.to_string())),
            ],
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(schema.core.len(), 2);
    }

    #[rstest]
    #[case("%Y-%m-%d")]
    #[case("%Y%m%d%H%M%S")]
    #[case("compact_datetime")]
    #[case("YYYY")]
    #[case("compact_date")]
    fn test_zerv_schema_new_valid_timestamp_patterns(#[case] pattern: &str) {
        let schema = ZervSchema::new(
            vec![Component::Var(Var::Timestamp(pattern.to_string()))],
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(schema.core.len(), 1);
    }

    // Error cases for ZervSchema::new()

    #[test]
    fn test_zerv_schema_new_error_empty_schema() {
        let result = ZervSchema::new(vec![], vec![], vec![]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("schema must contain at least one component")
        );
    }

    #[rstest]
    #[case("invalid_field", "unknown field 'invalid_field'")]
    #[case("unknown", "unknown field 'unknown'")]
    #[case("bad_field", "unknown field 'bad_field'")]
    #[case("custom.", "unknown field 'custom.'")]
    #[case("", "unknown field ''")]
    #[case("custom", "unknown field 'custom'")]
    fn test_zerv_schema_new_error_invalid_var_fields(
        #[case] field_name: &str,
        #[case] expected_error: &str,
    ) {
        let result = ZervSchema::new(
            vec![Component::Var(Var::Custom(field_name.to_string()))],
            vec![],
            vec![],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[rstest]
    #[case("INVALID_PATTERN", "unknown timestamp pattern 'INVALID_PATTERN'")]
    #[case("bad_pattern", "unknown timestamp pattern 'bad_pattern'")]
    #[case("YYYYMMDD", "unknown timestamp pattern 'YYYYMMDD'")]
    #[case("HHmmss", "unknown timestamp pattern 'HHmmss'")]
    #[case("invalid_format", "unknown timestamp pattern 'invalid_format'")]
    #[case("", "unknown timestamp pattern ''")]
    fn test_zerv_schema_new_error_invalid_timestamp_patterns(
        #[case] pattern: &str,
        #[case] expected_error: &str,
    ) {
        let result = ZervSchema::new(
            vec![Component::Var(Var::Timestamp(pattern.to_string()))],
            vec![],
            vec![],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[rstest]
    #[case(
        vec![Component::Var(Var::Custom("invalid_field".to_string()))],
        vec![],
        vec![],
        "unknown field 'invalid_field'"
    )]
    #[case(
        vec![],
        vec![Component::Var(Var::Custom("invalid_field".to_string()))],
        vec![],
        "unknown field 'invalid_field'"
    )]
    #[case(
        vec![],
        vec![],
        vec![Component::Var(Var::Custom("invalid_field".to_string()))],
        "unknown field 'invalid_field'"
    )]
    fn test_zerv_schema_new_error_invalid_in_different_sections(
        #[case] core: Vec<Component>,
        #[case] extra_core: Vec<Component>,
        #[case] build: Vec<Component>,
        #[case] expected_error: &str,
    ) {
        let result = ZervSchema::new(core, extra_core, build);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[test]
    fn test_zerv_schema_new_error_multiple_invalid_components() {
        let result = ZervSchema::new(
            vec![
                Component::Var(Var::Custom("invalid_field1".to_string())),
                Component::Var(Var::Custom("invalid_field2".to_string())),
            ],
            vec![],
            vec![],
        );
        assert!(result.is_err());
        // Should fail on the first invalid field
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unknown field 'invalid_field1'")
        );
    }

    #[test]
    fn test_zerv_schema_new_error_mixed_valid_invalid() {
        let result = ZervSchema::new(
            vec![
                Component::Var(Var::Major),
                Component::Var(Var::Custom("invalid_field".to_string())),
            ],
            vec![],
            vec![],
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unknown field 'invalid_field'")
        );
    }

    // Edge cases

    #[rstest]
    #[case("", 0)]
    #[case("测试", 6)]
    #[case("special-chars!@#$%", 18)]
    #[case("unicode_测试", 14)]
    fn test_zerv_schema_new_valid_string_components(
        #[case] string_value: &str,
        #[case] expected_len: usize,
    ) {
        let schema = ZervSchema::new(
            vec![Component::Str(string_value.to_string())],
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(schema.core.len(), 1);
        if let Component::Str(s) = &schema.core[0] {
            assert_eq!(s.len(), expected_len);
        }
    }

    #[rstest]
    #[case(0)]
    #[case(42)]
    #[case(1234567890)]
    #[case(u64::MAX)]
    fn test_zerv_schema_new_valid_integer_components(#[case] value: u64) {
        let schema = ZervSchema::new(vec![Component::Int(value)], vec![], vec![]).unwrap();
        assert_eq!(schema.core.len(), 1);
        if let Component::Int(i) = &schema.core[0] {
            assert_eq!(*i, value);
        }
    }

    #[rstest]
    #[case("custom.metadata.author")]
    #[case("custom.build.config.debug")]
    #[case("custom.nested.deep.field")]
    fn test_zerv_schema_new_valid_nested_custom_fields(#[case] field_name: &str) {
        let schema = ZervSchema::new(
            vec![Component::Var(Var::Custom(field_name.to_string()))],
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(schema.core.len(), 1);
    }
}
