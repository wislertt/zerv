use super::timestamp::resolve_timestamp;
use crate::version::zerv::{
    Component,
    Var,
    Zerv,
};

pub fn extract_core_values(zerv: &Zerv) -> Vec<u64> {
    let mut core_values = Vec::new();
    for comp in &zerv.schema.core {
        let val = match comp {
            Component::Var(var) => match var {
                Var::Major => zerv.vars.major.unwrap_or(0),
                Var::Minor => zerv.vars.minor.unwrap_or(0),
                Var::Patch => zerv.vars.patch.unwrap_or(0),
                Var::Timestamp(pattern) => zerv
                    .vars
                    .last_timestamp
                    .map(|ts| {
                        resolve_timestamp(pattern, ts)
                            .unwrap_or_else(|_| "0".to_string())
                            .parse()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0),
                _ => 0,
            },
            Component::Int(n) => *n,
            Component::Str(_) => 0,
        };
        core_values.push(val);
    }
    core_values
}
