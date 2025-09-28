use crate::version::zerv::{Component, ZervSchema};

/// Fixture for creating ZervSchema test data with RON string support
pub struct ZervSchemaFixture {
    schema: ZervSchema,
}

impl ZervSchemaFixture {
    /// Create a basic schema with core components
    pub fn basic() -> Self {
        Self {
            schema: ZervSchema::new(
                vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                vec![],
                vec![],
            )
            .unwrap_or_else(|e| panic!("Failed to create basic schema: {e}")),
        }
    }

    /// Create a schema with pre-release support
    pub fn with_pre_release() -> Self {
        let mut fixture = Self::basic();
        fixture
            .schema
            .extra_core
            .push(Component::VarField("pre_release".to_string()));
        fixture
    }

    /// Create a schema with epoch support
    pub fn with_epoch() -> Self {
        let mut fixture = Self::basic();
        fixture
            .schema
            .extra_core
            .push(Component::VarField("epoch".to_string()));
        fixture
    }

    /// Create a schema with post support
    pub fn with_post() -> Self {
        let mut fixture = Self::basic();
        fixture
            .schema
            .extra_core
            .push(Component::VarField("post".to_string()));
        fixture
    }

    /// Create a schema with dev support
    pub fn with_dev() -> Self {
        let mut fixture = Self::basic();
        fixture
            .schema
            .extra_core
            .push(Component::VarField("dev".to_string()));
        fixture
    }

    /// Create a schema with build metadata
    pub fn with_build(components: Vec<Component>) -> Self {
        let mut fixture = Self::basic();
        fixture.schema.build = components;
        fixture
    }

    /// Create a schema with custom extra core components
    pub fn with_extra_core(components: Vec<Component>) -> Self {
        let mut fixture = Self::basic();
        fixture.schema.extra_core = components;
        fixture
    }

    /// Create a complex schema with all components
    pub fn with_all_components() -> Self {
        Self {
            schema: ZervSchema::new(
                vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                vec![
                    Component::VarField("epoch".to_string()),
                    Component::VarField("pre_release".to_string()),
                    Component::VarField("post".to_string()),
                    Component::VarField("dev".to_string()),
                ],
                vec![
                    Component::String("build".to_string()),
                    Component::Integer(123),
                ],
            )
            .unwrap_or_else(|e| panic!("Failed to create complex schema: {e}")),
        }
    }

    /// Add a component to extra_core
    pub fn add_extra_core(mut self, component: Component) -> Self {
        self.schema.extra_core.push(component);
        self
    }

    /// Add a component to build
    pub fn add_build(mut self, component: Component) -> Self {
        self.schema.build.push(component);
        self
    }

    /// Get the schema
    pub fn schema(&self) -> &ZervSchema {
        &self.schema
    }

    /// Get the schema as RON string
    pub fn to_ron_string(&self) -> String {
        ron::ser::to_string_pretty(&self.schema, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|e| panic!("Failed to serialize schema to RON: {e}"))
    }

    /// Create from RON string
    pub fn from_ron_string(ron_string: &str) -> Result<Self, ron::error::SpannedError> {
        let schema: ZervSchema = ron::de::from_str(ron_string)?;
        Ok(Self { schema })
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
    fn test_basic_schema_fixture() {
        let fixture = ZervSchemaFixture::basic();
        let schema = fixture.schema();

        // Print the schema to see what it looks like
        println!("Basic ZervSchema:");
        println!("{schema:#?}");

        // Print the RON string representation
        let ron_string = fixture.to_ron_string();
        println!("\nBasic ZervSchema RON string:");
        println!("{ron_string}");

        // Verify the structure
        assert_eq!(schema.core.len(), 3);
        assert!(schema.extra_core.is_empty());
        assert!(schema.build.is_empty());

        // Verify core components
        assert!(matches!(schema.core[0], Component::VarField(ref name) if name == "major"));
        assert!(matches!(schema.core[1], Component::VarField(ref name) if name == "minor"));
        assert!(matches!(schema.core[2], Component::VarField(ref name) if name == "patch"));
    }
}
