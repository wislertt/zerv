mod calver;
mod standard;

use crate::schema::VersionSchema;
use crate::version::zerv::{
    ZervSchema,
    ZervVars,
};

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
    #[case("zerv-standard", ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(VersionSchema::StandardBase.schema()))]
    #[case("zerv-calver", ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(VersionSchema::CalverBase.schema()))]
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
