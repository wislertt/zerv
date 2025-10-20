use std::fmt::{
    Display,
    Formatter,
};

use serde::{
    Deserialize,
    Serialize,
};

use super::super::PrecedenceOrder;
use super::super::components::{
    Component,
    Var,
};
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

    // Convenience push methods
    pub fn push_core(&mut self, component: Component) -> Result<(), ZervError> {
        let mut current = self.core().clone();
        current.push(component);
        self.set_core(current)
    }

    pub fn push_extra_core(&mut self, component: Component) -> Result<(), ZervError> {
        let mut current = self.extra_core().clone();
        current.push(component);
        self.set_extra_core(current)
    }

    pub fn push_build(&mut self, component: Component) -> Result<(), ZervError> {
        let mut current = self.build().clone();
        current.push(component);
        self.set_build(current)
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
    pub fn pep440_default() -> Result<Self, ZervError> {
        Self::new(
            vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ],
            vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
                Component::Var(Var::Dev),
            ],
            vec![],
        )
    }

    pub fn semver_default() -> Result<Self, ZervError> {
        Self::new(
            vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ],
            vec![],
            vec![],
        )
    }

    pub fn pep440_based_precedence_order() -> PrecedenceOrder {
        PrecedenceOrder::pep440_based()
    }
}

impl Display for ZervSchema {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let ron_string = ron::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ron_string)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::super::super::PrecedenceOrder;
    use super::super::super::components::{
        Component,
        Var,
    };
    use super::ZervSchema;

    // Test constructors
    #[rstest]
    #[case(vec![Component::Var(Var::Major)], true)]
    #[case(vec![], false)]
    fn test_new(#[case] core: Vec<Component>, #[case] should_succeed: bool) {
        let result = ZervSchema::new(core, vec![], vec![]);
        assert_eq!(result.is_ok(), should_succeed);
    }

    #[rstest]
    #[case(vec![Component::Var(Var::Major)], PrecedenceOrder::pep440_based(), true)]
    #[case(vec![], PrecedenceOrder::default(), false)]
    fn test_new_with_precedence(
        #[case] core: Vec<Component>,
        #[case] precedence: PrecedenceOrder,
        #[case] should_succeed: bool,
    ) {
        let result = ZervSchema::new_with_precedence(core, vec![], vec![], precedence.clone());
        assert_eq!(result.is_ok(), should_succeed);
        if should_succeed {
            assert_eq!(result.unwrap().precedence_order(), &precedence);
        }
    }

    // Test getters
    #[test]
    fn test_getters() {
        let core = vec![Component::Var(Var::Major)];
        let extra_core = vec![Component::Var(Var::Epoch)];
        let build = vec![Component::Str("test".to_string())];
        let schema = ZervSchema::new(core.clone(), extra_core.clone(), build.clone()).unwrap();

        assert_eq!(schema.core(), &core);
        assert_eq!(schema.extra_core(), &extra_core);
        assert_eq!(schema.build(), &build);
        assert_eq!(schema.precedence_order(), &PrecedenceOrder::default());
    }

    // Test setters
    #[rstest]
    #[case("core", vec![Component::Var(Var::Major)], true)]
    #[case("core", vec![Component::Var(Var::Epoch)], false)]
    #[case("extra_core", vec![Component::Var(Var::Epoch)], true)]
    #[case("extra_core", vec![Component::Var(Var::Major)], false)]
    #[case("build", vec![Component::Var(Var::Distance)], true)]
    #[case("build", vec![Component::Var(Var::Major)], false)]
    fn test_setters(
        #[case] section: &str,
        #[case] components: Vec<Component>,
        #[case] should_succeed: bool,
    ) {
        let mut schema = ZervSchema::new(vec![Component::Var(Var::Minor)], vec![], vec![]).unwrap();
        let result = match section {
            "core" => schema.set_core(components),
            "extra_core" => schema.set_extra_core(components),
            "build" => schema.set_build(components),
            _ => panic!("Invalid section"),
        };
        assert_eq!(result.is_ok(), should_succeed);
    }

    #[test]
    fn test_set_precedence_order() {
        let mut schema = ZervSchema::new(vec![Component::Var(Var::Major)], vec![], vec![]).unwrap();
        let new_order = PrecedenceOrder::pep440_based();
        schema.set_precedence_order(new_order.clone());
        assert_eq!(schema.precedence_order(), &new_order);
    }

    // Test factory methods
    #[test]
    fn test_pep440_default() {
        let schema = ZervSchema::pep440_default().unwrap();
        assert_eq!(
            schema.core(),
            &vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch)
            ]
        );
        assert_eq!(
            schema.extra_core(),
            &vec![
                Component::Var(Var::Epoch),
                Component::Var(Var::PreRelease),
                Component::Var(Var::Post),
                Component::Var(Var::Dev)
            ]
        );
        assert_eq!(schema.build(), &vec![]);
    }

    #[test]
    fn test_semver_default() {
        let schema = ZervSchema::semver_default().unwrap();
        assert_eq!(
            schema.core(),
            &vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch)
            ]
        );
        assert_eq!(schema.extra_core(), &vec![]);
        assert_eq!(schema.build(), &vec![]);
    }

    #[test]
    fn test_pep440_based_precedence_order() {
        let order = ZervSchema::pep440_based_precedence_order();
        assert_eq!(order, PrecedenceOrder::pep440_based());
    }

    #[rstest]
    #[case::custom_schema(
        ZervSchema::new(
            vec![Component::Var(Var::Major), Component::Var(Var::Minor)],
            vec![Component::Var(Var::Epoch)],
            vec![],
        ).expect("Failed to create schema")
    )]
    #[case::standard_tier_1(ZervSchema::zerv_standard_tier_1())]
    fn test_to_string_roundtrip(#[case] original: ZervSchema) {
        let ron_string = original.to_string();
        let reconstructed: ZervSchema =
            ron::from_str(&ron_string).expect("Failed to reconstruct schema");

        assert_eq!(original, reconstructed);
    }
}
