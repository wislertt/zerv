use ron::de::from_str;
use serde::{
    Deserialize,
    Serialize,
};

use super::bump::precedence::{
    Precedence,
    PrecedenceOrder,
};
use super::components::ComponentConfig;
use crate::error::ZervError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaConfig {
    pub core: Vec<ComponentConfig>,
    pub extra_core: Vec<ComponentConfig>,
    pub build: Vec<ComponentConfig>,
    #[serde(default)]
    pub precedence_order: Vec<Precedence>,
}

impl From<SchemaConfig> for crate::version::zerv::schema::ZervSchema {
    fn from(config: SchemaConfig) -> Self {
        let precedence_order = if config.precedence_order.is_empty() {
            PrecedenceOrder::default()
        } else {
            PrecedenceOrder::from_precedences(config.precedence_order)
        };

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
            precedence_order,
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
            precedence_order: schema.precedence_order.to_vec(),
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
    use super::super::components::{
        Component,
        Var,
    };
    use super::*;
    use crate::version::zerv::schema::ZervSchema;

    #[test]
    fn test_schema_config_to_zerv_schema_conversion() {
        let schema_config = SchemaConfig {
            core: vec![
                ComponentConfig::Var { field: Var::Major },
                ComponentConfig::Var { field: Var::Minor },
            ],
            extra_core: vec![ComponentConfig::Var { field: Var::Patch }],
            build: vec![ComponentConfig::Str {
                value: "build".to_string(),
            }],
            precedence_order: vec![],
        };

        let zerv_schema: ZervSchema = schema_config.into();

        assert_eq!(zerv_schema.core.len(), 2);
        assert_eq!(zerv_schema.extra_core.len(), 1);
        assert_eq!(zerv_schema.build.len(), 1);

        assert!(matches!(zerv_schema.core[0], Component::Var(Var::Major)));
        assert!(matches!(
            zerv_schema.extra_core[0],
            Component::Var(Var::Patch)
        ));
        assert!(matches!(zerv_schema.build[0], Component::Str(_)));
    }

    #[test]
    fn test_parse_ron_schema() {
        let ron_schema = r#"
            SchemaConfig(
                core: [
                    Var(field: Major),
                    Var(field: Minor),
                ],
                extra_core: [],
                build: [
                    Str(value: "build_id")
                ]
            )
        "#;

        let schema = parse_ron_schema(ron_schema).unwrap();
        assert_eq!(schema.core.len(), 2);
        assert_eq!(schema.extra_core.len(), 0);
        assert_eq!(schema.build.len(), 1);
        // Should use default precedence order
        assert_eq!(schema.precedence_order.len(), 11);
    }

    #[test]
    fn test_parse_ron_schema_with_precedence() {
        use crate::version::zerv::bump::precedence::Precedence;

        let ron_schema = r#"
            SchemaConfig(
                core: [
                    Var(field: Major),
                    Var(field: Minor),
                ],
                extra_core: [],
                build: [
                    Str(value: "build_id")
                ],
                precedence_order: [
                    Major,
                    Minor,
                    Patch,
                    Core,
                    Build
                ]
            )
        "#;

        let schema = parse_ron_schema(ron_schema).unwrap();
        assert_eq!(schema.core.len(), 2);
        assert_eq!(schema.extra_core.len(), 0);
        assert_eq!(schema.build.len(), 1);
        assert_eq!(schema.precedence_order.len(), 5);
        assert_eq!(
            schema.precedence_order.get_precedence(0),
            Some(&Precedence::Major)
        );
        assert_eq!(
            schema.precedence_order.get_precedence(1),
            Some(&Precedence::Minor)
        );
        assert_eq!(
            schema.precedence_order.get_precedence(4),
            Some(&Precedence::Build)
        );
    }
}
