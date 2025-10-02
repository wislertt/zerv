mod calver;
mod standard;

pub use calver::{get_calver_schema, zerv_calver_tier_1, zerv_calver_tier_2, zerv_calver_tier_3};
pub use standard::{
    get_standard_schema, zerv_standard_tier_1, zerv_standard_tier_2, zerv_standard_tier_3,
};

use crate::constants::ron_fields;
use crate::version::zerv::{Component, ZervSchema, ZervVars};

fn determine_tier(vars: &ZervVars) -> u8 {
    if vars.dirty.unwrap_or(false) {
        3 // Dirty
    } else if vars.distance.unwrap_or(0) > 0 {
        2 // Distance, clean
    } else {
        1 // Tagged, clean
    }
}

fn tier_1_core() -> Vec<Component> {
    vec![
        Component::VarField(ron_fields::MAJOR.to_string()),
        Component::VarField(ron_fields::MINOR.to_string()),
        Component::VarField(ron_fields::PATCH.to_string()),
    ]
}

fn tier_1_extra_core() -> Vec<Component> {
    vec![
        Component::VarField(ron_fields::EPOCH.to_string()),
        Component::VarField(ron_fields::PRE_RELEASE.to_string()),
        Component::VarField(ron_fields::POST.to_string()),
    ]
}

fn tier_2_build() -> Vec<Component> {
    vec![
        Component::VarField(ron_fields::BRANCH.to_string()),
        Component::VarField(ron_fields::COMMIT_HASH_SHORT.to_string()),
    ]
}

fn tier_3_extra_core() -> Vec<Component> {
    vec![
        Component::VarField(ron_fields::EPOCH.to_string()),
        Component::VarField(ron_fields::PRE_RELEASE.to_string()),
        Component::VarField(ron_fields::POST.to_string()),
        Component::VarField(ron_fields::DEV.to_string()),
    ]
}

fn tier_3_build() -> Vec<Component> {
    vec![
        Component::VarField(ron_fields::BRANCH.to_string()),
        Component::VarField(ron_fields::DISTANCE.to_string()),
        Component::VarField(ron_fields::COMMIT_HASH_SHORT.to_string()),
    ]
}

pub fn get_preset_schema(name: &str, vars: &ZervVars) -> Option<ZervSchema> {
    match name {
        "zerv-standard" => Some(get_standard_schema(vars)),
        "zerv-calver" => Some(get_calver_schema(vars)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::ZervVars;
    use rstest::rstest;

    #[rstest]
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, 1)]
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, 2)]
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, 3)]
    #[case(ZervVars { dirty: Some(true), distance: Some(10), ..Default::default() }, 3)]
    fn test_tier_determination(#[case] vars: ZervVars, #[case] expected_tier: u8) {
        assert_eq!(determine_tier(&vars), expected_tier);
    }

    #[rstest]
    #[case("zerv-standard", ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(zerv_standard_tier_1()))]
    #[case("zerv-calver", ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(zerv_calver_tier_1()))]
    #[case("unknown", ZervVars::default(), None)]
    fn test_get_preset_schema(
        #[case] name: &str,
        #[case] vars: ZervVars,
        #[case] expected: Option<ZervSchema>,
    ) {
        let schema = get_preset_schema(name, &vars);
        assert_eq!(schema, expected);
    }
}
