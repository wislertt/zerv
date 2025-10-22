mod calver;
mod standard;

use crate::version::zerv::{
    Component,
    Var,
    ZervSchema,
    ZervVars,
};

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
        Component::Var(Var::Major),
        Component::Var(Var::Minor),
        Component::Var(Var::Patch),
    ]
}

fn tier_1_extra_core() -> Vec<Component> {
    vec![
        Component::Var(Var::Epoch),
        Component::Var(Var::PreRelease),
        Component::Var(Var::Post),
    ]
}

fn tier_2_build() -> Vec<Component> {
    vec![
        Component::Var(Var::BumpedBranch),
        Component::Var(Var::Distance),
        Component::Var(Var::BumpedCommitHashShort),
    ]
}

fn tier_3_extra_core() -> Vec<Component> {
    vec![
        Component::Var(Var::Epoch),
        Component::Var(Var::PreRelease),
        Component::Var(Var::Post),
        Component::Var(Var::Dev),
    ]
}

fn tier_3_build() -> Vec<Component> {
    tier_2_build()
}

pub fn get_preset_schema(name: &str, vars: &ZervVars) -> Option<ZervSchema> {
    match name {
        "zerv-standard" => Some(ZervSchema::get_standard_schema(vars)),
        "zerv-calver" => Some(ZervSchema::get_calver_schema(vars)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::zerv::ZervVars;

    #[rstest]
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, 1)]
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, 2)]
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, 3)]
    #[case(ZervVars { dirty: Some(true), distance: Some(10), ..Default::default() }, 3)]
    fn test_tier_determination(#[case] vars: ZervVars, #[case] expected_tier: u8) {
        assert_eq!(determine_tier(&vars), expected_tier);
    }

    #[rstest]
    #[case("zerv-standard", ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(ZervSchema::zerv_standard_tier_1()))]
    #[case("zerv-calver", ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(ZervSchema::zerv_calver_tier_1()))]
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
