use crate::version::zerv::bump::precedence::PrecedenceOrder;
use crate::version::zerv::{
    Component,
    Var,
    ZervSchema,
};

/// Fixture for creating ZervSchema test data using presets
pub struct ZervSchemaFixture {
    schema: ZervSchema,
}

impl ZervSchemaFixture {
    /// Create a new fixture with standard tier 1 schema (major.minor.patch)
    pub fn new() -> Self {
        Self {
            schema: ZervSchema::zerv_standard_tier_1(),
        }
    }

    /// Build and return the final ZervSchema
    pub fn build(self) -> ZervSchema {
        self.schema
    }

    /// Create standard tier 1 schema (major.minor.patch)
    pub fn standard_tier_1() -> Self {
        use crate::schema::VersionSchema;
        Self {
            schema: VersionSchema::StandardBasePrereleasePost.create_schema(),
        }
    }

    /// Create standard tier 2 schema (with build metadata)
    pub fn standard_tier_2() -> Self {
        Self {
            schema: ZervSchema::zerv_standard_tier_2(),
        }
    }

    /// Create standard tier 3 schema (with dev components)
    pub fn standard_tier_3() -> Self {
        Self {
            schema: ZervSchema::zerv_standard_tier_3(),
        }
    }

    /// Create calver tier 1 schema
    pub fn calver_tier_1() -> Self {
        Self {
            schema: ZervSchema::zerv_calver_tier_1(),
        }
    }

    /// Create calver tier 2 schema
    pub fn calver_tier_2() -> Self {
        Self {
            schema: ZervSchema::zerv_calver_tier_2(),
        }
    }

    /// Create calver tier 3 schema
    pub fn calver_tier_3() -> Self {
        Self {
            schema: ZervSchema::zerv_calver_tier_3(),
        }
    }

    pub fn empty() -> Self {
        Self {
            schema: ZervSchema::new_with_precedence(
                vec![Component::Var(Var::Major)], // At least one component required
                vec![],
                vec![],
                PrecedenceOrder::default(),
            )
            .expect("Failed to create base schema"),
        }
    }

    pub fn with_core(mut self, components: Vec<Component>) -> Self {
        self.schema
            .set_core(components)
            .expect("Failed to set core components");
        self
    }

    pub fn with_extra_core(mut self, components: Vec<Component>) -> Self {
        self.schema
            .set_extra_core(components)
            .expect("Failed to set extra core components");
        self
    }

    pub fn with_build(mut self, components: Vec<Component>) -> Self {
        self.schema
            .set_build(components)
            .expect("Failed to set build components");
        self
    }

    pub fn with_precedence(mut self, precedence_order: PrecedenceOrder) -> Self {
        self.schema.set_precedence_order(precedence_order);
        self
    }

    pub fn with_major_minor_patch(self) -> Self {
        self.with_core(vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ])
    }

    pub fn with_epoch_in_extra_core(self) -> Self {
        self.with_extra_core(vec![Component::Var(Var::Epoch)])
    }

    pub fn with_major_minor(self) -> Self {
        self.with_core(vec![Component::Var(Var::Major), Component::Var(Var::Minor)])
    }

    pub fn with_prerelease_in_extra_core(self) -> Self {
        self.with_extra_core(vec![Component::Var(Var::PreRelease)])
    }
}

impl Default for ZervSchemaFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ZervSchemaFixture> for ZervSchema {
    fn from(fixture: ZervSchemaFixture) -> Self {
        fixture.schema
    }
}

impl From<ZervSchema> for ZervSchemaFixture {
    fn from(schema: ZervSchema) -> Self {
        Self { schema }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::zerv::bump::precedence::PrecedenceOrder;
    use crate::version::zerv::components::{
        Component,
        Var,
    };

    #[test]
    fn test_new_schema_fixture() {
        let schema = ZervSchemaFixture::new().build();

        // Verify the structure
        assert_eq!(schema.core().len(), 3);
        assert!(!schema.extra_core().is_empty()); // standard_tier_1 has extra_core
        assert!(schema.build().is_empty());
    }

    #[test]
    fn test_preset_constructors() {
        let tier1 = ZervSchemaFixture::standard_tier_1().build();
        let tier2 = ZervSchemaFixture::standard_tier_2().build();
        let tier3 = ZervSchemaFixture::standard_tier_3().build();

        // All should have core components
        assert_eq!(tier1.core().len(), 3);
        assert_eq!(tier2.core().len(), 3);
        assert_eq!(tier3.core().len(), 3);

        // Tier 2 and 3 should have build components
        assert!(tier1.build().is_empty());
        assert!(!tier2.build().is_empty());
        assert!(!tier3.build().is_empty());
    }

    // Builder pattern tests

    #[rstest]
    #[case::empty(
        ZervSchemaFixture::empty().build(),
        vec![Component::Var(Var::Major)],
        vec![],
        vec![]
    )]
    #[case::with_core(
        ZervSchemaFixture::empty()
            .with_core(vec![
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ])
            .build(),
        vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ],
        vec![],
        vec![]
    )]
    #[case::with_extra_core(
        ZervSchemaFixture::empty()
            .with_extra_core(vec![Component::Var(Var::Epoch)])
            .build(),
        vec![Component::Var(Var::Major)],
        vec![Component::Var(Var::Epoch)],
        vec![]
    )]
    #[case::with_build(
        ZervSchemaFixture::empty()
            .with_build(vec![Component::Var(Var::Distance)])
            .build(),
        vec![Component::Var(Var::Major)],
        vec![],
        vec![Component::Var(Var::Distance)]
    )]
    #[case::with_major_minor_patch(
        ZervSchemaFixture::empty()
            .with_major_minor_patch()
            .build(),
        vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ],
        vec![],
        vec![]
    )]
    #[case::with_major_minor(
        ZervSchemaFixture::empty()
            .with_major_minor()
            .build(),
        vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
        ],
        vec![],
        vec![]
    )]
    #[case::with_epoch_in_extra_core(
        ZervSchemaFixture::empty()
            .with_epoch_in_extra_core()
            .build(),
        vec![Component::Var(Var::Major)],
        vec![Component::Var(Var::Epoch)],
        vec![]
    )]
    #[case::with_prerelease_in_extra_core(
        ZervSchemaFixture::empty()
            .with_prerelease_in_extra_core()
            .build(),
        vec![Component::Var(Var::Major)],
        vec![Component::Var(Var::PreRelease)],
        vec![]
    )]
    fn test_builder_methods(
        #[case] schema: crate::version::zerv::ZervSchema,
        #[case] expected_core: Vec<Component>,
        #[case] expected_extra_core: Vec<Component>,
        #[case] expected_build: Vec<Component>,
    ) {
        assert_eq!(schema.core(), &expected_core);
        assert_eq!(schema.extra_core(), &expected_extra_core);
        assert_eq!(schema.build(), &expected_build);
    }

    #[rstest]
    #[case::pep440_based(PrecedenceOrder::pep440_based())]
    #[case::default(PrecedenceOrder::default())]
    fn test_with_precedence(#[case] precedence_order: PrecedenceOrder) {
        let schema = ZervSchemaFixture::empty()
            .with_precedence(precedence_order.clone())
            .build();

        assert_eq!(schema.precedence_order(), &precedence_order);
    }

    #[rstest]
    #[case::fluency_test(
        ZervSchemaFixture::empty()
            .with_major_minor_patch()
            .with_epoch_in_extra_core()
            .with_build(vec![Component::Var(Var::Distance)])
            .with_precedence(PrecedenceOrder::pep440_based())
            .build(),
        3, // expected_core_len
        1, // expected_extra_core_len
        1, // expected_build_len
        Some(PrecedenceOrder::pep440_based()) // expected_precedence
    )]
    #[case::to_string_test(
        ZervSchemaFixture::empty()
            .with_major_minor()
            .with_epoch_in_extra_core()
            .build(),
        2, // expected_core_len
        1, // expected_extra_core_len
        0, // expected_build_len
        None // expected_precedence (use default)
    )]
    fn test_builder_complex_scenarios(
        #[case] schema: crate::version::zerv::ZervSchema,
        #[case] expected_core_len: usize,
        #[case] expected_extra_core_len: usize,
        #[case] expected_build_len: usize,
        #[case] expected_precedence: Option<PrecedenceOrder>,
    ) {
        // Verify component counts
        assert_eq!(schema.core().len(), expected_core_len);
        assert_eq!(schema.extra_core().len(), expected_extra_core_len);
        assert_eq!(schema.build().len(), expected_build_len);

        // Verify precedence if specified
        if let Some(expected) = expected_precedence {
            assert_eq!(*schema.precedence_order(), expected);
        }

        // Test round-trip serialization
        let ron_string = schema.to_string();
        let parsed: crate::version::zerv::ZervSchema =
            ron::from_str(&ron_string).expect("Failed to parse RON string");

        assert_eq!(schema.core(), parsed.core());
        assert_eq!(schema.extra_core(), parsed.extra_core());
        assert_eq!(schema.build(), parsed.build());
        assert_eq!(schema.precedence_order(), parsed.precedence_order());
    }
}
