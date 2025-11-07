use crate::schema::VersionSchema;
use crate::version::zerv::{
    ZervSchema,
    ZervVars,
};

pub fn get_preset_schema(name: &str, vars: &ZervVars) -> Option<ZervSchema> {
    tracing::debug!("Loading preset schema: {}", name);

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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::schema::{
        VersionSchema,
        schema_names,
    };
    use crate::version::zerv::ZervVars;

    #[rstest]
    #[case(schema_names::STANDARD, ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(VersionSchema::StandardBase.schema()))]
    #[case(schema_names::CALVER, ZervVars { dirty: Some(false), distance: Some(0), ..Default::default() }, Some(VersionSchema::CalverBase.schema()))]
    #[case("zerv-standard", ZervVars::default(), None)] // No longer supported
    #[case("zerv-calver", ZervVars::default(), None)] // No longer supported
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
