use super::super::components::{
    Component,
    Var,
};
use super::core::ZervSchema;
use crate::error::ZervError;
use crate::utils::constants::timestamp_patterns;

impl ZervSchema {
    // Main validation entry point
    pub fn validate(&self) -> Result<(), ZervError> {
        // Check that schema has at least one component
        if self.core().is_empty() && self.extra_core().is_empty() && self.build().is_empty() {
            return Err(ZervError::StdinError(
                "Invalid Zerv RON: schema must contain at least one component in core, extra_core, or build sections".to_string()
            ));
        }

        // Section-specific validation
        self.validate_core()?;
        self.validate_extra_core()?;
        self.validate_build()?;

        Ok(())
    }

    // Validate core section
    fn validate_core(&self) -> Result<(), ZervError> {
        Self::validate_components(self.core())?;
        let seen_primary = self.validate_core_placement()?;
        self.validate_primary_order(&seen_primary)?;
        Ok(())
    }

    fn validate_core_placement(&self) -> Result<Vec<Var>, ZervError> {
        let mut seen_primary = Vec::new();

        for component in self.core() {
            if let Component::Var(var) = component {
                if var.is_primary_component() {
                    if seen_primary.contains(var) {
                        return Err(ZervError::StdinError(format!(
                            "Duplicate primary component: {var:?}"
                        )));
                    }
                    seen_primary.push(var.clone());
                } else if var.is_secondary_component() {
                    return Err(ZervError::StdinError(format!(
                        "Secondary component {var:?} must be in extra_core section"
                    )));
                }
            }
        }

        Ok(seen_primary)
    }

    fn validate_primary_order(&self, seen_primary: &[Var]) -> Result<(), ZervError> {
        if seen_primary.len() <= 1 {
            return Ok(());
        }

        let order_map = Var::primary_component_order();
        let indices: Vec<usize> = seen_primary
            .iter()
            .filter_map(|var| order_map.get_index_of(var))
            .collect();

        for i in 1..indices.len() {
            if indices[i] <= indices[i - 1] {
                return Err(ZervError::StdinError(
                    "Primary components must be in order: major → minor → patch".to_string(),
                ));
            }
        }

        Ok(())
    }

    // Validate extra_core section
    fn validate_extra_core(&self) -> Result<(), ZervError> {
        use std::collections::HashSet;

        // Existing component validation
        Self::validate_components(self.extra_core())?;

        // Component placement validation
        let mut seen_secondary = HashSet::new();

        for component in self.extra_core() {
            if let Component::Var(var) = component {
                if var.is_secondary_component() {
                    if !seen_secondary.insert(var.clone()) {
                        return Err(ZervError::StdinError(format!(
                            "Duplicate secondary component: {var:?}"
                        )));
                    }
                } else if var.is_primary_component() {
                    return Err(ZervError::StdinError(format!(
                        "Primary component {var:?} must be in core section"
                    )));
                }
                // Context components allowed anywhere
            }
        }

        Ok(())
    }

    // Validate build section
    fn validate_build(&self) -> Result<(), ZervError> {
        // Existing component validation
        Self::validate_components(self.build())?;

        // Component placement validation
        for component in self.build() {
            if let Component::Var(var) = component {
                if var.is_primary_component() {
                    return Err(ZervError::StdinError(format!(
                        "Primary component {var:?} must be in core section"
                    )));
                } else if var.is_secondary_component() {
                    return Err(ZervError::StdinError(format!(
                        "Secondary component {var:?} must be in extra_core section"
                    )));
                }
                // Context components allowed in build
            }
        }

        Ok(())
    }

    pub fn validate_components(components: &[Component]) -> Result<(), ZervError> {
        for component in components {
            Self::validate_component(component)?;
        }
        Ok(())
    }

    pub fn validate_component(component: &Component) -> Result<(), ZervError> {
        match component {
            Component::Var(var) => {
                if let Var::Timestamp(pattern) = var
                    && !Self::is_valid_timestamp_pattern(pattern)
                {
                    let valid_patterns = timestamp_patterns::get_valid_timestamp_patterns();
                    return Err(ZervError::StdinError(format!(
                        "Invalid Zerv RON: unknown timestamp pattern '{pattern}' in ts() component. Valid patterns are: {} or custom format starting with %",
                        valid_patterns.join(", ")
                    )));
                }
            }
            Component::Str(_) => {}
            Component::UInt(_) => {}
        }
        Ok(())
    }

    fn is_valid_timestamp_pattern(pattern: &str) -> bool {
        // Check preset patterns
        if timestamp_patterns::get_valid_timestamp_patterns().contains(&pattern) {
            return true;
        }

        // Check for custom chrono format strings (start with %)
        pattern.starts_with('%')
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::super::super::components::{
        Component,
        Var,
    };
    use super::ZervSchema;
    use crate::test_utils::ZervSchemaFixture;

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
        let component = Component::UInt(value);
        assert!(ZervSchema::validate_component(&component).is_ok());
    }

    // Test validate_components function
    #[test]
    fn test_validate_components_empty() {
        assert!(ZervSchema::validate_components(&[]).is_ok());
    }

    #[rstest]
    #[case(vec![Component::Var(Var::Major), Component::Str("test".to_string()), Component::UInt(42)], true)]
    #[case(vec![Component::Var(Var::Major), Component::Var(Var::Timestamp("INVALID".to_string()))], false)]
    fn test_validate_components(#[case] components: Vec<Component>, #[case] should_succeed: bool) {
        let result = ZervSchema::validate_components(&components);
        assert_eq!(result.is_ok(), should_succeed);
        if !should_succeed {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("unknown timestamp pattern")
            );
        }
    }

    // Test is_valid_timestamp_pattern function
    #[rstest]
    #[case("YYYY", true)]
    #[case("MM", true)]
    #[case("%Y-%m-%d", true)]
    #[case("%custom", true)]
    #[case("INVALID", false)]
    #[case("not_a_pattern", false)]
    #[case("", false)]
    fn test_is_valid_timestamp_pattern(#[case] pattern: &str, #[case] expected: bool) {
        assert_eq!(ZervSchema::is_valid_timestamp_pattern(pattern), expected);
    }

    // Test main validate function
    #[test]
    fn test_validate_empty_schema() {
        let result = ZervSchema::new(vec![], vec![], vec![]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("schema must contain at least one component")
        );
    }

    #[test]
    fn test_validate_valid_schema() {
        let schema = ZervSchemaFixture::new().build();
        assert!(schema.validate().is_ok());
    }

    // Test validation errors using ZervSchema::new which calls validate
    #[rstest]
    #[case(vec![Component::Var(Var::Major), Component::Var(Var::Major)], "Duplicate primary component")]
    #[case(vec![Component::Var(Var::Minor), Component::Var(Var::Major)], "Primary components must be in order")]
    #[case(vec![Component::Var(Var::Epoch)], "must be in extra_core section")]
    fn test_validate_core_errors(#[case] core: Vec<Component>, #[case] expected_error: &str) {
        let result = ZervSchema::new(core, vec![], vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[rstest]
    #[case(vec![Component::Var(Var::Epoch), Component::Var(Var::Epoch)], "Duplicate secondary component")]
    #[case(vec![Component::Var(Var::Major)], "must be in core section")]
    fn test_validate_extra_core_errors(
        #[case] extra_core: Vec<Component>,
        #[case] expected_error: &str,
    ) {
        let result = ZervSchema::new(vec![Component::Var(Var::Minor)], extra_core, vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    #[rstest]
    #[case(vec![Component::Var(Var::Major)], "must be in core section")]
    #[case(vec![Component::Var(Var::Epoch)], "must be in extra_core section")]
    fn test_validate_build_errors(#[case] build: Vec<Component>, #[case] expected_error: &str) {
        let result = ZervSchema::new(vec![Component::Var(Var::Minor)], vec![], build);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }

    // Test context components allowed anywhere
    #[rstest]
    #[case(Var::Distance)]
    #[case(Var::Dirty)]
    #[case(Var::BumpedBranch)]
    #[case(Var::Custom("test".to_string()))]
    fn test_context_components_allowed_in_any_section(#[case] var: Var) {
        let component = Component::Var(var);

        // In core
        assert!(ZervSchema::new(vec![component.clone()], vec![], vec![]).is_ok());

        // In extra_core
        assert!(
            ZervSchema::new(
                vec![Component::Var(Var::Major)],
                vec![component.clone()],
                vec![]
            )
            .is_ok()
        );

        // In build
        assert!(ZervSchema::new(vec![Component::Var(Var::Major)], vec![], vec![component]).is_ok());
    }

    #[test]
    fn test_validate_valid_sections() {
        // Valid core with primary components in order
        assert!(
            ZervSchema::new(
                vec![Component::Var(Var::Major), Component::Var(Var::Minor)],
                vec![],
                vec![]
            )
            .is_ok()
        );

        // Valid extra_core with secondary components
        assert!(
            ZervSchema::new(
                vec![Component::Var(Var::Major)],
                vec![Component::Var(Var::Epoch), Component::Var(Var::Post)],
                vec![]
            )
            .is_ok()
        );

        // Valid build with context components
        assert!(
            ZervSchema::new(
                vec![Component::Var(Var::Major)],
                vec![],
                vec![
                    Component::Var(Var::Distance),
                    Component::Str("test".to_string())
                ]
            )
            .is_ok()
        );
    }
}
