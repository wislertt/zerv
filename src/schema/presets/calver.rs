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
        VersionSchema::Calver.schema_with_zerv(vars)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::schema::{
        SchemaContextExt,
        VersionSchema,
    };
    use crate::version::zerv::ZervVars;

    #[rstest]
    // Clean tagged (base only) -> CalverBase with smart context (no context for clean)
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, VersionSchema::CalverBase.schema())]
    // Distance -> CalverBasePrereleasePost with smart context (context added for distance)
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, VersionSchema::CalverBasePrereleasePost.schema().with_build_context())]
    // Dirty -> CalverBasePrereleasePostDev with smart context (context added for dirty)
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, VersionSchema::CalverBasePrereleasePostDev.schema().with_build_context())]
    fn test_get_calver_schema(#[case] vars: ZervVars, #[case] expected_schema: ZervSchema) {
        let schema = ZervSchema::get_calver_schema(&vars);
        assert_eq!(schema, expected_schema);
    }
}
