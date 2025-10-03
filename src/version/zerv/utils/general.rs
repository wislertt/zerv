use super::timestamp::resolve_timestamp;
use crate::constants::ron_fields;
use crate::version::zerv::{Component, Zerv};

pub fn extract_core_values(zerv: &Zerv) -> Vec<u64> {
    let mut core_values = Vec::new();
    for comp in &zerv.schema.core {
        let val = match comp {
            Component::VarField(field) => match field.as_str() {
                ron_fields::MAJOR => zerv.vars.major.unwrap_or(0),
                ron_fields::MINOR => zerv.vars.minor.unwrap_or(0),
                ron_fields::PATCH => zerv.vars.patch.unwrap_or(0),
                _ => 0,
            },
            Component::VarTimestamp(pattern) => zerv
                .vars
                .last_timestamp
                .map(|ts| {
                    resolve_timestamp(pattern, ts)
                        .unwrap_or_else(|_| "0".to_string())
                        .parse()
                        .unwrap_or(0)
                })
                .unwrap_or(0),
            Component::Integer(n) => *n,
            _ => 0,
        };
        core_values.push(val);
    }
    core_values
}
