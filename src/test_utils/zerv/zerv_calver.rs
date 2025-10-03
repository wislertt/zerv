use crate::constants::ron_fields;
use crate::version::zerv::{Component, Zerv, ZervSchema, ZervVars};

/// CalVer helper functions (demonstrating VarTimestamp usage)
pub fn calver_yy_mm_patch() -> Zerv {
    Zerv {
        schema: ZervSchema {
            core: vec![
                Component::VarTimestamp("YY".to_string()),
                Component::VarTimestamp("MM".to_string()),
                Component::VarField(ron_fields::PATCH.to_string()),
            ],
            extra_core: vec![],
            build: vec![],
        },
        vars: ZervVars {
            patch: Some(5),
            last_timestamp: Some(1710547200), // 2024-03-15
            ..Default::default()
        },
    }
}

pub fn calver_yyyy_mm_patch() -> Zerv {
    Zerv {
        schema: ZervSchema {
            core: vec![
                Component::VarTimestamp("YYYY".to_string()),
                Component::VarTimestamp("MM".to_string()),
                Component::VarField(ron_fields::PATCH.to_string()),
            ],
            extra_core: vec![],
            build: vec![],
        },
        vars: ZervVars {
            patch: Some(1),
            last_timestamp: Some(1710547200),
            ..Default::default()
        },
    }
}

pub fn calver_with_timestamp_build() -> Zerv {
    Zerv {
        schema: ZervSchema {
            core: vec![
                Component::VarField(ron_fields::MAJOR.to_string()),
                Component::VarField(ron_fields::MINOR.to_string()),
                Component::VarField(ron_fields::PATCH.to_string()),
            ],
            extra_core: vec![],
            build: vec![
                Component::VarTimestamp("YYYY".to_string()),
                Component::VarTimestamp("MM".to_string()),
                Component::VarTimestamp("DD".to_string()),
            ],
        },
        vars: ZervVars {
            major: Some(1),
            minor: Some(0),
            patch: Some(0),
            last_timestamp: Some(1710547200),
            ..Default::default()
        },
    }
}
