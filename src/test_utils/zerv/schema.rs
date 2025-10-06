use crate::schema::{
    zerv_calver_tier_1,
    zerv_calver_tier_2,
    zerv_calver_tier_3,
    zerv_standard_tier_1,
    zerv_standard_tier_2,
    zerv_standard_tier_3,
};
use crate::version::zerv::ZervSchema;

/// Fixture for creating ZervSchema test data using presets
pub struct ZervSchemaFixture {
    schema: ZervSchema,
}

impl ZervSchemaFixture {
    /// Create a new fixture with standard tier 1 schema (major.minor.patch)
    pub fn new() -> Self {
        Self {
            schema: zerv_standard_tier_1(),
        }
    }

    /// Build and return the final ZervSchema
    pub fn build(self) -> ZervSchema {
        self.schema
    }

    /// Create standard tier 1 schema (major.minor.patch)
    pub fn standard_tier_1() -> Self {
        Self {
            schema: zerv_standard_tier_1(),
        }
    }

    /// Create standard tier 2 schema (with build metadata)
    pub fn standard_tier_2() -> Self {
        Self {
            schema: zerv_standard_tier_2(),
        }
    }

    /// Create standard tier 3 schema (with dev components)
    pub fn standard_tier_3() -> Self {
        Self {
            schema: zerv_standard_tier_3(),
        }
    }

    /// Create calver tier 1 schema
    pub fn calver_tier_1() -> Self {
        Self {
            schema: zerv_calver_tier_1(),
        }
    }

    /// Create calver tier 2 schema
    pub fn calver_tier_2() -> Self {
        Self {
            schema: zerv_calver_tier_2(),
        }
    }

    /// Create calver tier 3 schema
    pub fn calver_tier_3() -> Self {
        Self {
            schema: zerv_calver_tier_3(),
        }
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
    use super::*;

    #[test]
    fn test_new_schema_fixture() {
        let schema = ZervSchemaFixture::new().build();

        // Verify the structure
        assert_eq!(schema.core.len(), 3);
        assert!(!schema.extra_core.is_empty()); // standard_tier_1 has extra_core
        assert!(schema.build.is_empty());
    }

    #[test]
    fn test_preset_constructors() {
        let tier1 = ZervSchemaFixture::standard_tier_1().build();
        let tier2 = ZervSchemaFixture::standard_tier_2().build();
        let tier3 = ZervSchemaFixture::standard_tier_3().build();

        // All should have core components
        assert_eq!(tier1.core.len(), 3);
        assert_eq!(tier2.core.len(), 3);
        assert_eq!(tier3.core.len(), 3);

        // Tier 2 and 3 should have build components
        assert!(tier1.build.is_empty());
        assert!(!tier2.build.is_empty());
        assert!(!tier3.build.is_empty());
    }
}
