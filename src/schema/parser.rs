use crate::error::ZervError;
use crate::version::zerv::{Component, ZervSchema};
use ron::de::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SchemaConfig {
    pub core: Vec<ComponentConfig>,
    pub extra_core: Vec<ComponentConfig>,
    pub build: Vec<ComponentConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
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

impl From<SchemaConfig> for ZervSchema {
    fn from(config: SchemaConfig) -> Self {
        ZervSchema {
            core: config.core.into_iter().map(Component::from).collect(),
            extra_core: config.extra_core.into_iter().map(Component::from).collect(),
            build: config.build.into_iter().map(Component::from).collect(),
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
                    (type: "VarField", field: "major"),
                    (type: "VarField", field: "minor"),
                    (type: "VarField", field: "patch"),
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
                    (type: "VarTimestamp", pattern: "YYYY"),
                    (type: "VarTimestamp", pattern: "MM"),
                    (type: "VarField", field: "patch"),
                ],
                extra_core: [
                    (type: "VarField", field: "post"),
                ],
                build: [
                    (type: "String", value: "build"),
                    (type: "Integer", value: 123),
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
