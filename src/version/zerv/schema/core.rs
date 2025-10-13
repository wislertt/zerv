use serde::{
    Deserialize,
    Serialize,
};

use super::super::PrecedenceOrder;
use super::super::components::Component;
use crate::error::ZervError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZervSchema {
    core: Vec<Component>,
    extra_core: Vec<Component>,
    build: Vec<Component>,
    #[serde(default)]
    precedence_order: PrecedenceOrder,
}

impl ZervSchema {
    // Getters
    pub fn core(&self) -> &Vec<Component> {
        &self.core
    }

    pub fn extra_core(&self) -> &Vec<Component> {
        &self.extra_core
    }

    pub fn build(&self) -> &Vec<Component> {
        &self.build
    }

    pub fn precedence_order(&self) -> &PrecedenceOrder {
        &self.precedence_order
    }

    // Setters with validation
    pub fn set_core(&mut self, core: Vec<Component>) -> Result<(), ZervError> {
        Self::validate_components(&core)?;
        let temp_schema = Self {
            core: core.clone(),
            extra_core: self.extra_core.clone(),
            build: self.build.clone(),
            precedence_order: self.precedence_order.clone(),
        };
        temp_schema.validate()?;
        self.core = core;
        Ok(())
    }

    pub fn set_extra_core(&mut self, extra_core: Vec<Component>) -> Result<(), ZervError> {
        Self::validate_components(&extra_core)?;
        let temp_schema = Self {
            core: self.core.clone(),
            extra_core: extra_core.clone(),
            build: self.build.clone(),
            precedence_order: self.precedence_order.clone(),
        };
        temp_schema.validate()?;
        self.extra_core = extra_core;
        Ok(())
    }

    pub fn set_build(&mut self, build: Vec<Component>) -> Result<(), ZervError> {
        Self::validate_components(&build)?;
        let temp_schema = Self {
            core: self.core.clone(),
            extra_core: self.extra_core.clone(),
            build: build.clone(),
            precedence_order: self.precedence_order.clone(),
        };
        temp_schema.validate()?;
        self.build = build;
        Ok(())
    }

    pub fn set_precedence_order(&mut self, precedence_order: PrecedenceOrder) {
        self.precedence_order = precedence_order;
    }

    // Constructors
    pub fn new(
        core: Vec<Component>,
        extra_core: Vec<Component>,
        build: Vec<Component>,
    ) -> Result<Self, ZervError> {
        Self::new_with_precedence(core, extra_core, build, PrecedenceOrder::default())
    }

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

    // Factory methods
    pub fn pep440_based_precedence_order() -> PrecedenceOrder {
        PrecedenceOrder::pep440_based()
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
    #[case(Var::Major)]
    #[case(Var::Minor)]
    #[case(Var::Patch)]
    #[case(Var::Epoch)]
    #[case(Var::PreRelease)]
    #[case(Var::Post)]
    #[case(Var::Dev)]
    #[case(Var::Distance)]
    #[case(Var::Dirty)]
    #[case(Var::BumpedBranch)]
    #[case(Var::BumpedCommitHashShort)]
    #[case(Var::LastBranch)]
    #[case(Var::LastCommitHash)]
    #[case(Var::LastTimestamp)]
    #[case(Var::Custom("build_id".to_string()))]
    #[case(Var::Custom("environment".to_string()))]
    #[case(Var::Custom("metadata.author".to_string()))]
    fn test_validate_component_valid_var_field(#[case] var: Var) {
        let is_secondary = var.is_secondary_component();
        let component = Component::Var(var);
        if is_secondary {
            assert!(ZervSchema::new(vec![], vec![component], vec![]).is_ok());
        } else {
            assert!(ZervSchema::new(vec![component], vec![], vec![]).is_ok());
        }
    }

    #[test]
    fn test_validate_component_custom_fields_always_valid() {
        let test_cases = vec![
            "build_id",
            "environment",
            "custom.field",
            "any_name",
            "",
            "123",
        ];

        for field_name in test_cases {
            let component = Component::Var(Var::Custom(field_name.to_string()));
            assert!(ZervSchema::new(vec![component], vec![], vec![]).is_ok());
        }
    }

    #[test]
    fn test_zerv_schema_new_with_validation() {
        let schema = ZervSchema::new(vec![Component::Var(Var::Major)], vec![], vec![]).unwrap();
        assert_eq!(schema.core().len(), 1);
    }

    #[test]
    fn test_zerv_schema_new_invalid() {
        let result = ZervSchema::new(vec![], vec![], vec![]);
        assert!(result.is_err());
    }

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
}
