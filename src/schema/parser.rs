use crate::error::ZervError;
use crate::version::zerv::{Component, ZervSchema};
use ron::de::from_str;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub core: Vec<ComponentConfig>,
    pub extra_core: Vec<ComponentConfig>,
    pub build: Vec<ComponentConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentConfig {
    String { value: String },
    Integer { value: u64 },
    VarField { field: String },
    VarTimestamp { pattern: String },
}

impl From<ComponentConfig> for Component {
    fn from(config: ComponentConfig) -> Self {
        match config {
            ComponentConfig::String { value } => Component::String(value),
            ComponentConfig::Integer { value } => Component::Integer(value),
            ComponentConfig::VarField { field } => Component::VarField(field),
            ComponentConfig::VarTimestamp { pattern } => Component::VarTimestamp(pattern),
        }
    }
}

impl From<&Component> for ComponentConfig {
    fn from(component: &Component) -> Self {
        match component {
            Component::String(value) => ComponentConfig::String {
                value: value.clone(),
            },
            Component::Integer(value) => ComponentConfig::Integer { value: *value },
            Component::VarField(field) => ComponentConfig::VarField {
                field: field.clone(),
            },
            Component::VarTimestamp(pattern) => ComponentConfig::VarTimestamp {
                pattern: pattern.clone(),
            },
        }
    }
}

impl From<SchemaConfig> for ZervSchema {
    fn from(config: SchemaConfig) -> Self {
        ZervSchema {
            core: config.core.into_iter().map(Component::from).collect(),
            extra_core: config.extra_core.into_iter().map(Component::from).collect(),
            build: config.build.into_iter().map(Component::from).collect(),
        }
    }
}

impl From<ZervSchema> for SchemaConfig {
    fn from(schema: ZervSchema) -> Self {
        SchemaConfig {
            core: schema
                .core
                .into_iter()
                .map(|c| ComponentConfig::from(&c))
                .collect(),
            extra_core: schema
                .extra_core
                .into_iter()
                .map(|c| ComponentConfig::from(&c))
                .collect(),
            build: schema
                .build
                .into_iter()
                .map(|c| ComponentConfig::from(&c))
                .collect(),
        }
    }
}

pub fn parse_ron_schema(ron_str: &str) -> Result<ZervSchema, ZervError> {
    let config: SchemaConfig = from_str(ron_str)
        .map_err(|e| ZervError::SchemaParseError(format!("Invalid RON schema: {e}")))?;
    Ok(config.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
        let ron_str = r#"
            SchemaConfig(
                core: [
                    VarField(field: "major"),
                    VarField(field: "minor"),
                    VarField(field: "patch"),
                ],
                extra_core: [],
                build: []
            )
        "#;

        let schema = parse_ron_schema(ron_str).unwrap();
        assert_eq!(schema.core.len(), 3);
        assert_eq!(schema.extra_core.len(), 0);
        assert_eq!(schema.build.len(), 0);
    }

    #[test]
    fn test_parse_complex_schema() {
        let ron_str = r#"
            SchemaConfig(
                core: [
                    VarTimestamp(pattern: "YYYY"),
                    VarTimestamp(pattern: "MM"),
                    VarField(field: "patch"),
                ],
                extra_core: [
                    VarField(field: "post"),
                ],
                build: [
                    String(value: "build"),
                    Integer(value: 123),
                ]
            )
        "#;

        let schema = parse_ron_schema(ron_str).unwrap();
        assert_eq!(schema.core.len(), 3);
        assert_eq!(schema.extra_core.len(), 1);
        assert_eq!(schema.build.len(), 2);
    }

    #[test]
    fn test_parse_invalid_schema() {
        let invalid_ron = "invalid ron syntax";
        let result = parse_ron_schema(invalid_ron);
        assert!(matches!(result, Err(ZervError::SchemaParseError(_))));
    }

    #[test]
    fn test_component_config_conversion() {
        let string_config = ComponentConfig::String {
            value: "test".to_string(),
        };
        let component: Component = string_config.into();
        assert!(matches!(component, Component::String(_)));

        let integer_config = ComponentConfig::Integer { value: 42 };
        let component: Component = integer_config.into();
        assert!(matches!(component, Component::Integer(42)));

        let var_field_config = ComponentConfig::VarField {
            field: "major".to_string(),
        };
        let component: Component = var_field_config.into();
        assert!(matches!(component, Component::VarField(_)));

        let var_timestamp_config = ComponentConfig::VarTimestamp {
            pattern: "YYYY".to_string(),
        };
        let component: Component = var_timestamp_config.into();
        assert!(matches!(component, Component::VarTimestamp(_)));
    }
}
