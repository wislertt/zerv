use super::determine_tier;
use crate::schema::VersionSchema;
use crate::version::zerv::{
    ZervSchema,
    ZervVars,
};

impl ZervSchema {
    // Tier 1: Tagged, clean - YYYY-MM-DD-PATCH
    pub fn zerv_calver_tier_1() -> Self {
        VersionSchema::CalverBasePrerelease.schema()
    }

    // Tier 2: Distance, clean - YYYY-MM-DD-PATCH.post<distance>+branch.<commit>
    pub fn zerv_calver_tier_2() -> Self {
        VersionSchema::CalverBasePrereleasePostContext.schema()
    }

    // Tier 3: Dirty - YYYY-MM-DD-PATCH.dev<timestamp>+branch.<distance>.<commit>
    pub fn zerv_calver_tier_3() -> Self {
        VersionSchema::CalverBasePrereleasePostDevContext.schema()
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
