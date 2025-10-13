use std::str::FromStr;

use crate::error::ZervError;
use crate::version::zerv::schema::ZervSchema;

impl FromStr for ZervSchema {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::de::from_str(s).map_err(|e| ZervError::StdinError(format!("Invalid RON schema: {e}")))
    }
}

pub fn parse_ron_schema(ron_str: &str) -> Result<ZervSchema, ZervError> {
    ron_str.parse()
}

#[cfg(test)]
mod tests {
    use super::super::bump::precedence::{
        Precedence,
        PrecedenceOrder,
    };
    use super::super::components::{
        Component,
        Var,
    };
    use super::*;

    #[test]
    fn test_parse_ron_schema() {
        let ron_schema = r#"
            ZervSchema(
                core: [
                    var(Major),
                    var(Minor),
                ],
                extra_core: [],
                build: [
                    str("build_id")
                ]
            )
        "#;

        let expected_schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major), Component::Var(Var::Minor)],
            vec![],
            vec![Component::Str("build_id".to_string())],
            PrecedenceOrder::default(),
        )
        .unwrap();

        let schema: ZervSchema = ron_schema.parse().unwrap();
        assert_eq!(schema, expected_schema);
    }

    #[test]
    fn test_parse_ron_schema_with_precedence() {
        let ron_schema = r#"
            ZervSchema(
                core: [
                    var(Major),
                    var(Minor),
                ],
                extra_core: [],
                build: [
                    str("build_id")
                ],
                precedence_order: [
                    Major,
                    Minor,
                    Patch,
                    Core,
                    Build,
                ]
            )
        "#;

        let expected_schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major), Component::Var(Var::Minor)],
            vec![],
            vec![Component::Str("build_id".to_string())],
            PrecedenceOrder::from_precedences(vec![
                Precedence::Major,
                Precedence::Minor,
                Precedence::Patch,
                Precedence::Core,
                Precedence::Build,
            ]),
        )
        .unwrap();

        let schema: ZervSchema = ron_schema.parse().unwrap();
        assert_eq!(schema, expected_schema);
    }

    #[test]
    fn test_parse_ron_schema_with_empty_precedence() {
        let ron_schema = r#"
            ZervSchema(
                core: [
                    var(Major),
                    var(Minor),
                ],
                extra_core: [],
                build: [
                    str("build_id")
                ],
                precedence_order: []
            )
        "#;

        let expected_schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major), Component::Var(Var::Minor)],
            vec![],
            vec![Component::Str("build_id".to_string())],
            PrecedenceOrder::from_precedences(vec![]),
        )
        .unwrap();

        let schema: ZervSchema = ron_schema.parse().unwrap();
        assert_eq!(schema, expected_schema);
    }
}
