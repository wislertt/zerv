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
        let zerv = Zerv::new(schema, vars).unwrap();

        let display_output = zerv.to_string();

        assert!(display_output.contains("schema"));
        assert!(display_output.contains("vars"));
        assert!(display_output.contains("major"));
        assert!(display_output.contains("minor"));
    }

    #[test]
    fn test_zerv_display_roundtrip() {
        let schema = ZervSchema {
            core: vec![Component::Var(Var::Major)],
            extra_core: vec![],
            build: vec![],
            precedence_order: PrecedenceOrder::default(),
        };
        let vars = ZervVars {
            major: Some(1),
            ..Default::default()
        };
        let original = Zerv::new(schema, vars).unwrap();

        let ron_string = original.to_string();
        let parsed: Zerv = ron::de::from_str(&ron_string).unwrap();
        assert_eq!(original, parsed);
    }
}
