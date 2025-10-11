use std::str::FromStr;

use crate::error::ZervError;
use crate::version::zerv::Zerv;

impl FromStr for Zerv {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::de::from_str(s)
            .map_err(|e| ZervError::InvalidVersion(format!("Invalid Zerv RON format: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::bump::precedence::PrecedenceOrder;
    use crate::version::zerv::vars::ZervVars;
    use crate::version::zerv::{
        Component,
        Var,
        ZervSchema,
    };

    #[test]
    fn test_zerv_parse_simple() {
        let ron_str = r#"
            (
                schema: (
                    core: [var(Major)],
                    extra_core: [],
                    build: [],
                ),
                vars: (
                    major: Some(1),
                    minor: None,
                    patch: None,
                    epoch: None,
                    pre_release: None,
                    post: None,
                    dev: None,
                    last_timestamp: None,
                    last_branch: None,
                    bumped_branch: None,
                    distance: None,
                    dirty: None,
                    last_commit_hash: None,
                    bumped_commit_hash: None,
                    custom: {},
                ),
            )
        "#;

        let expected = Zerv {
            schema: ZervSchema {
                core: vec![Component::Var(Var::Major)],
                extra_core: vec![],
                build: vec![],
                precedence_order: PrecedenceOrder::default(),
            },
            vars: ZervVars {
                major: Some(1),
                custom: serde_json::json!({}),
                ..Default::default()
            },
        };

        let parsed: Zerv = ron_str.parse().unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_zerv_parse_invalid() {
        let invalid_ron = "invalid ron syntax";
        let result: Result<Zerv, _> = invalid_ron.parse();
        assert!(result.is_err());
        assert!(matches!(result, Err(ZervError::InvalidVersion(_))));
    }

    #[test]
    fn test_zerv_parse_roundtrip() {
        let schema = ZervSchema {
            core: vec![
                Component::Var(Var::Major),
                Component::Str(".".to_string()),
                Component::Var(Var::Minor),
            ],
            extra_core: vec![],
            build: vec![],
            precedence_order: PrecedenceOrder::default(),
        };
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            ..Default::default()
        };
        let original = Zerv::new(schema, vars).unwrap();

        let ron_string = original.to_string();
        let parsed: Zerv = ron_string.parse().unwrap();
        assert_eq!(original, parsed);
    }
}
