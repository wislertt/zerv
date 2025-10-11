use crate::version::zerv::bump::precedence::PrecedenceOrder;
use crate::version::zerv::{
    Component,
    Var,
    Zerv,
    ZervSchema,
    ZervVars,
};

/// CalVer helper functions (demonstrating VarTimestamp usage)
pub fn calver_yy_mm_patch() -> Zerv {
    Zerv {
        schema: ZervSchema {
            core: vec![
                Component::Var(Var::Timestamp("YY".to_string())),
                Component::Var(Var::Timestamp("MM".to_string())),
                Component::Var(Var::Patch),
            ],
            extra_core: vec![],
            build: vec![],
            precedence_order: PrecedenceOrder::default(),
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
                Component::Var(Var::Timestamp("YYYY".to_string())),
                Component::Var(Var::Timestamp("MM".to_string())),
                Component::Var(Var::Patch),
            ],
            extra_core: vec![],
            build: vec![],
            precedence_order: PrecedenceOrder::default(),
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
                Component::Var(Var::Major),
                Component::Var(Var::Minor),
                Component::Var(Var::Patch),
            ],
            extra_core: vec![],
            build: vec![
                Component::Var(Var::Timestamp("YYYY".to_string())),
                Component::Var(Var::Timestamp("MM".to_string())),
                Component::Var(Var::Timestamp("DD".to_string())),
            ],
            precedence_order: PrecedenceOrder::default(),
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
