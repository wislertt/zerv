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
        // Existing component validation
        Self::validate_components(self.core())?;

        // Component placement validation
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
                // Context components allowed anywhere
            }
        }

        // Check primary component order: major → minor → patch
        if seen_primary.len() > 1 {
            let order_map = Var::primary_component_order();

            let mut indices = Vec::new();
            for var in &seen_primary {
                if let Some(index) = order_map.get_index_of(var) {
                    indices.push(index);
                }
            }

            // Check indices are increasing
            for i in 1..indices.len() {
                if indices[i] <= indices[i - 1] {
                    return Err(ZervError::StdinError(
                        "Primary components must be in order: major → minor → patch".to_string(),
                    ));
                }
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
            Component::Int(_) => {}
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
}
