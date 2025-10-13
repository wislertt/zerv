use super::{
    determine_tier,
    tier_1_core,
    tier_1_extra_core,
    tier_2_build,
    tier_3_build,
    tier_3_extra_core,
};
use crate::version::zerv::bump::precedence::PrecedenceOrder;
use crate::version::zerv::{
    ZervSchema,
    ZervVars,
};

// Tier 1: Tagged, clean - major.minor.patch
pub fn zerv_standard_tier_1() -> ZervSchema {
    ZervSchema::new_with_precedence(
        tier_1_core(),
        tier_1_extra_core(),
        vec![],
        PrecedenceOrder::default(),
    )
    .unwrap()
}

// Tier 2: Distance, clean - major.minor.patch.post<distance>+branch.<commit>
pub fn zerv_standard_tier_2() -> ZervSchema {
    ZervSchema::new_with_precedence(
        tier_1_core(),
        tier_1_extra_core(),
        tier_2_build(),
        PrecedenceOrder::default(),
    )
    .unwrap()
}

// Tier 3: Dirty - major.minor.patch.dev<timestamp>+branch.<distance>.<commit>
pub fn zerv_standard_tier_3() -> ZervSchema {
    ZervSchema::new_with_precedence(
        tier_1_core(),
        tier_3_extra_core(),
        tier_3_build(),
        PrecedenceOrder::default(),
    )
    .unwrap()
}

pub fn get_standard_schema(vars: &ZervVars) -> ZervSchema {
    let tier = determine_tier(vars);
    match tier {
        1 => zerv_standard_tier_1(),
        2 => zerv_standard_tier_2(),
        3 => zerv_standard_tier_3(),
        _ => unreachable!("Invalid tier"),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::version::zerv::ZervVars;

    #[rstest]
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, zerv_standard_tier_1())]
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, zerv_standard_tier_2())]
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, zerv_standard_tier_3())]
    fn test_get_standard_schema(#[case] vars: ZervVars, #[case] expected_schema: ZervSchema) {
        let schema = get_standard_schema(&vars);
        assert_eq!(schema, expected_schema);
    }
}
