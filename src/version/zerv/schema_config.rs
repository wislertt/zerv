use super::components::ComponentConfig;
use crate::error::ZervError;
use ron::de::from_str;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub core: Vec<ComponentConfig>,
    pub extra_core: Vec<ComponentConfig>,
    pub build: Vec<ComponentConfig>,
}

impl From<SchemaConfig> for crate::version::zerv::schema::ZervSchema {
    fn from(config: SchemaConfig) -> Self {
        crate::version::zerv::schema::ZervSchema {
            core: config
                .core
                .into_iter()
                .map(super::components::Component::from)
                .collect(),
            extra_core: config
                .extra_core
                .into_iter()
                .map(super::components::Component::from)
                .collect(),
            build: config
                .build
                .into_iter()
                .map(super::components::Component::from)
                .collect(),
        }
    }
}

impl From<&crate::version::zerv::schema::ZervSchema> for SchemaConfig {
    fn from(schema: &crate::version::zerv::schema::ZervSchema) -> Self {
        SchemaConfig {
            core: schema
                .core
                .iter()
                .map(super::components::ComponentConfig::from)
                .collect(),
            extra_core: schema
                .extra_core
                .iter()
                .map(super::components::ComponentConfig::from)
                .collect(),
            build: schema
                .build
                .iter()
                .map(super::components::ComponentConfig::from)
                .collect(),
        }
    }
}

pub fn parse_ron_schema(
    ron_str: &str,
) -> Result<crate::version::zerv::schema::ZervSchema, ZervError> {
    let schema_config: SchemaConfig =
        from_str(ron_str).map_err(|e| ZervError::StdinError(format!("Invalid RON schema: {e}")))?;

    Ok(schema_config.into())
}

#[cfg(test)]
mod tests {
    use super::super::components::Component;
    use super::*;
    use crate::version::zerv::schema::ZervSchema;

    #[test]
    fn test_schema_config_to_zerv_schema_conversion() {
        let schema_config = SchemaConfig {
            core: vec![
                ComponentConfig::VarField {
                    field: "major".to_string(),
                },
                ComponentConfig::VarField {
                    field: "minor".to_string(),
                },
            ],
            extra_core: vec![ComponentConfig::VarField {
                field: "patch".to_string(),
            }],
            build: vec![ComponentConfig::String {
                value: "build".to_string(),
            }],
        };

        let zerv_schema: ZervSchema = schema_config.into();

        assert_eq!(zerv_schema.core.len(), 2);
        assert_eq!(zerv_schema.extra_core.len(), 1);
        assert_eq!(zerv_schema.build.len(), 1);

        assert!(matches!(zerv_schema.core[0], Component::VarField(_)));
        assert!(matches!(zerv_schema.extra_core[0], Component::VarField(_)));
        assert!(matches!(zerv_schema.build[0], Component::String(_)));
    }

    #[test]
    fn test_parse_ron_schema() {
        let ron_schema = r#"
            SchemaConfig(
                core: [
                    VarField(field: "major"),
                    VarField(field: "minor"),
                ],
                extra_core: [],
                build: [
                    String(value: "build_id")
                ]
            )
        "#;

        let schema = parse_ron_schema(ron_schema).unwrap();
        assert_eq!(schema.core.len(), 2);
        assert_eq!(schema.extra_core.len(), 0);
        assert_eq!(schema.build.len(), 1);
    }
}
