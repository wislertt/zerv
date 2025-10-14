use super::{
    determine_tier,
    tier_1_extra_core,
    tier_2_build,
    tier_3_build,
    tier_3_extra_core,
};
use crate::utils::constants::timestamp_patterns;
use crate::version::zerv::bump::precedence::PrecedenceOrder;
use crate::version::zerv::{
    Component,
    Var,
    ZervSchema,
    ZervVars,
};

impl ZervSchema {
    // Tier 1: Tagged, clean - YYYY-MM-DD-PATCH
    pub fn zerv_calver_tier_1() -> Self {
        Self::new_with_precedence(
            vec![
                Component::Var(Var::Timestamp(timestamp_patterns::YYYY.to_string())),
                Component::Var(Var::Timestamp(timestamp_patterns::MM.to_string())),
                Component::Var(Var::Timestamp(timestamp_patterns::DD.to_string())),
                Component::Var(Var::Patch),
            ],
            tier_1_extra_core(),
            vec![],
            PrecedenceOrder::default(),
        )
        .unwrap()
    }

    // Tier 2: Distance, clean - YYYY-MM-DD-PATCH.post<distance>+branch.<commit>
    pub fn zerv_calver_tier_2() -> Self {
        Self::new_with_precedence(
            vec![
                Component::Var(Var::Timestamp(timestamp_patterns::YYYY.to_string())),
                Component::Var(Var::Timestamp(timestamp_patterns::MM.to_string())),
                Component::Var(Var::Timestamp(timestamp_patterns::DD.to_string())),
                Component::Var(Var::Patch),
            ],
            tier_1_extra_core(),
            tier_2_build(),
            PrecedenceOrder::default(),
        )
        .unwrap()
    }

    // Tier 3: Dirty - YYYY-MM-DD-PATCH.dev<timestamp>+branch.<distance>.<commit>
    pub fn zerv_calver_tier_3() -> Self {
        Self::new_with_precedence(
            vec![
                Component::Var(Var::Timestamp(timestamp_patterns::YYYY.to_string())),
                Component::Var(Var::Timestamp(timestamp_patterns::MM.to_string())),
                Component::Var(Var::Timestamp(timestamp_patterns::DD.to_string())),
                Component::Var(Var::Patch),
            ],
            tier_3_extra_core(),
            tier_3_build(),
            PrecedenceOrder::default(),
        )
        .unwrap()
    }

    pub fn get_calver_schema(vars: &ZervVars) -> Self {
        let tier = determine_tier(vars);
        match tier {
            1 => Self::zerv_calver_tier_1(),
            2 => Self::zerv_calver_tier_2(),
            3 => Self::zerv_calver_tier_3(),
            _ => unreachable!("Invalid tier"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::zerv::ZervVars;

    #[rstest]
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, ZervSchema::zerv_calver_tier_1())]
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, ZervSchema::zerv_calver_tier_2())]
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, ZervSchema::zerv_calver_tier_3())]
    fn test_get_calver_schema(#[case] vars: ZervVars, #[case] expected_schema: ZervSchema) {
        let schema = ZervSchema::get_calver_schema(&vars);
        assert_eq!(schema, expected_schema);
    }
}
