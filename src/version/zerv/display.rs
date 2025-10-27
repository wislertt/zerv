use std::fmt;

use crate::version::zerv::Zerv;

impl fmt::Display for Zerv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(ron_string) => write!(f, "{ron_string}"),
            Err(_) => write!(f, "Error serializing Zerv to RON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::bump::precedence::PrecedenceOrder;
    use crate::version::zerv::{
        Component,
        Var,
        ZervSchema,
        ZervVars,
    };

    #[test]
    fn test_zerv_display() {
        let schema = ZervSchema::new_with_precedence(
            vec![
                Component::Var(Var::Major),
                Component::Str(".".to_string()),
                Component::Var(Var::Minor),
            ],
            vec![],
            vec![],
            PrecedenceOrder::default(),
        )
        .unwrap();
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            ..Default::default()
        };
        let zerv = Zerv::new(schema, vars).unwrap();

        let display_output = zerv.to_string();

        assert!(display_output.contains("schema"));
        assert!(display_output.contains("vars"));
        assert!(display_output.contains("major"));
        assert!(display_output.contains("minor"));
    }

    #[test]
    fn test_zerv_display_roundtrip() {
        let schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major)],
            vec![],
            vec![],
            PrecedenceOrder::default(),
        )
        .unwrap();
        let vars = ZervVars {
            major: Some(1),
            ..Default::default()
        };
        let original = Zerv::new(schema, vars).unwrap();

        let ron_string = original.to_string();
        let parsed: Zerv = ron::de::from_str(&ron_string).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_zerv_display_always_produces_output() {
        // Even with all None values, display should not panic
        let schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major)],
            vec![],
            vec![],
            PrecedenceOrder::default(),
        )
        .unwrap();
        let vars = ZervVars::default();
        let zerv = Zerv::new(schema, vars).unwrap();

        let display_output = zerv.to_string();
        // Should produce some output, not panic
        assert!(!display_output.is_empty());
    }
}
