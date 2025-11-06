mod calver;
mod standard;

use crate::schema::VersionSchema;
use crate::version::zerv::{
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

pub fn get_preset_schema(name: &str, vars: &ZervVars) -> Option<ZervSchema> {
    tracing::debug!("Loading preset schema: {}", name);

    // Handle old schema names for backward compatibility with deprecation warnings
    match name {
        "zerv-standard" => {
            tracing::warn!("Schema 'zerv-standard' is deprecated. Use 'standard' instead");
            tracing::debug!("Using built-in zerv-standard schema (legacy)");
            Some(ZervSchema::get_standard_schema(vars))
        }
        "zerv-calver" => {
            tracing::warn!("Schema 'zerv-calver' is deprecated. Use 'calver' instead");
            tracing::debug!("Using built-in zerv-calver schema (legacy)");
            Some(ZervSchema::get_calver_schema(vars))
        }
        // Old tier-based schemas with deprecation warnings and mapping
        "zerv_standard_tier_1" => {
            tracing::warn!(
                "Schema 'zerv_standard_tier_1' is deprecated. Use 'standard-base-prerelease' instead"
            );
            Some(ZervSchema::zerv_standard_tier_1())
        }
        "zerv_standard_tier_2" => {
            tracing::warn!(
                "Schema 'zerv_standard_tier_2' is deprecated. Use 'standard-base-prerelease-post' instead"
            );
            Some(ZervSchema::zerv_standard_tier_2())
        }
        "zerv_standard_tier_3" => {
            tracing::warn!(
                "Schema 'zerv_standard_tier_3' is deprecated. Use 'standard-base-prerelease-post-dev' instead"
            );
            Some(ZervSchema::zerv_standard_tier_3())
        }
        "zerv_calver_tier_1" => {
            tracing::warn!(
                "Schema 'zerv_calver_tier_1' is deprecated. Use 'calver-base-prerelease' instead"
            );
            Some(ZervSchema::zerv_calver_tier_1())
        }
        "zerv_calver_tier_2" => {
            tracing::warn!(
                "Schema 'zerv_calver_tier_2' is deprecated. Use 'calver-base-prerelease-post' instead"
            );
            Some(ZervSchema::zerv_calver_tier_2())
        }
        "zerv_calver_tier_3" => {
            tracing::warn!(
                "Schema 'zerv_calver_tier_3' is deprecated. Use 'calver-base-prerelease-post-dev' instead"
            );
            Some(ZervSchema::zerv_calver_tier_3())
        }
        _ => {
            // Try to parse as new flexible schema
            match name.parse::<VersionSchema>() {
                Ok(schema) => {
                    tracing::debug!("Using flexible schema: {}", name);
                    Some(schema.schema_with_zerv(vars))
                }
                Err(_) => {
                    tracing::warn!("Unknown preset schema name: {}", name);
                    None
                }
            }
        }
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
    #[case(
        "zerv_standard_tier_1",
        ZervVars::default(),
        Some(ZervSchema::zerv_standard_tier_1())
    )]
    #[case(
        "zerv_standard_tier_2",
        ZervVars::default(),
        Some(ZervSchema::zerv_standard_tier_2())
    )]
    #[case(
        "zerv_standard_tier_3",
        ZervVars::default(),
        Some(ZervSchema::zerv_standard_tier_3())
    )]
    #[case(
        "zerv_calver_tier_1",
        ZervVars::default(),
        Some(ZervSchema::zerv_calver_tier_1())
    )]
    #[case(
        "zerv_calver_tier_2",
        ZervVars::default(),
        Some(ZervSchema::zerv_calver_tier_2())
    )]
    #[case(
        "zerv_calver_tier_3",
        ZervVars::default(),
        Some(ZervSchema::zerv_calver_tier_3())
    )]
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
