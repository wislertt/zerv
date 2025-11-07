use crate::schema::VersionSchema;
use crate::version::zerv::{
    ZervSchema,
    ZervVars,
};

impl ZervSchema {
    // Tier 1: Tagged, clean - major.minor.patch
    pub fn zerv_standard_tier_1() -> Self {
        VersionSchema::StandardBasePrereleasePost.schema()
    }

    // Tier 2: Distance, clean - major.minor.patch.post<distance>+branch.<commit>
    pub fn zerv_standard_tier_2() -> Self {
        VersionSchema::StandardBasePrereleasePostContext.schema()
    }

    // Tier 3: Dirty - major.minor.patch.dev<timestamp>+branch.<distance>.<commit>
    pub fn zerv_standard_tier_3() -> Self {
        VersionSchema::StandardBasePrereleasePostDevContext.schema()
    }

    pub fn get_standard_schema(vars: &ZervVars) -> Self {
        VersionSchema::Standard.schema_with_zerv(vars)
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
    // Clean tagged (base only) -> StandardBase with smart context (no context for clean)
    #[case(ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, VersionSchema::StandardBase.schema())]
    // Distance -> StandardBasePrereleasePost with smart context (context added for distance)
    #[case(ZervVars { dirty: Some(false), distance: Some(5), ..Default::default() }, VersionSchema::StandardBasePrereleasePost.schema().with_build_context())]
    // Dirty -> StandardBasePrereleasePostDev with smart context (context added for dirty)
    #[case(ZervVars { dirty: Some(true), distance: Some(0), ..Default::default() }, VersionSchema::StandardBasePrereleasePostDev.schema().with_build_context())]
    fn test_get_standard_schema(#[case] vars: ZervVars, #[case] expected_schema: ZervSchema) {
        let schema = ZervSchema::get_standard_schema(&vars);
        assert_eq!(schema, expected_schema);
    }
}
