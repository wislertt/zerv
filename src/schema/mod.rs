mod parser;
mod presets;
mod validation;

pub use parser::{ComponentConfig, SchemaConfig, parse_ron_schema};
pub use presets::{
    get_calver_schema, get_preset_schema, get_standard_schema, zerv_calver_tier_1,
    zerv_calver_tier_2, zerv_calver_tier_3, zerv_standard_tier_1, zerv_standard_tier_2,
    zerv_standard_tier_3,
};
pub use validation::{component_validation, structure_validation};

use crate::error::ZervError;
use crate::version::zerv::{Zerv, ZervVars};

pub fn create_zerv_version(
    vars: ZervVars,
    schema_name: Option<&str>,
    schema_ron: Option<&str>,
) -> Result<Zerv, ZervError> {
    let schema = match (schema_name, schema_ron) {
        // Error if both are provided
        (Some(_), Some(_)) => {
            return Err(ZervError::ConflictingSchemas(
                "Cannot specify both schema_name and schema_ron".to_string(),
            ));
        }

        // Custom RON schema
        (None, Some(ron_str)) => parse_ron_schema(ron_str)?,

        // Built-in schema
        (Some(name), None) => {
            if let Some(schema) = get_preset_schema(name, &vars) {
                schema
            } else {
                return Err(ZervError::UnknownSchema(name.to_string()));
            }
        }

        // Neither provided - use default
        (None, None) => get_preset_schema("zerv-standard", &vars).unwrap(),
    };

    Ok(Zerv { schema, vars })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::{ZervSchema, ZervVars};
    use rstest::rstest;

    #[rstest]
    #[case(
        "zerv-standard",
        ZervVars {
            major: Some(1), minor: Some(2), patch: Some(3),
            dirty: Some(false), distance: Some(0),
            ..Default::default()
        },
        zerv_standard_tier_1()
    )]
    #[case(
        "zerv-standard",
        ZervVars {
            major: Some(1), minor: Some(2), patch: Some(3),
            dirty: Some(false), distance: Some(5), post: Some(5),
            bumped_branch: Some("main".to_string()), bumped_commit_hash: Some("abc123".to_string()),
            ..Default::default()
        },
        zerv_standard_tier_2()
    )]
    #[case(
        "zerv-standard",
        ZervVars {
            major: Some(1), minor: Some(2), patch: Some(3),
            dirty: Some(true), dev: Some(1234567890),
            bumped_branch: Some("feature".to_string()), bumped_commit_hash: Some("def456".to_string()),
            ..Default::default()
        },
        zerv_standard_tier_3()
    )]
    #[case(
        "zerv-calver",
        ZervVars {
            patch: Some(1), dirty: Some(false), distance: Some(0),
            last_timestamp: Some(1710547200),
            ..Default::default()
        },
        zerv_calver_tier_1()
    )]
    #[case(
        "zerv-calver",
        ZervVars {
            patch: Some(1), dirty: Some(false), distance: Some(5), post: Some(5),
            bumped_branch: Some("main".to_string()), bumped_commit_hash: Some("abc123".to_string()),
            last_timestamp: Some(1710547200),
            ..Default::default()
        },
        zerv_calver_tier_2()
    )]
    #[case(
        "zerv-calver",
        ZervVars {
            patch: Some(1), dirty: Some(true), dev: Some(1234567890),
            bumped_branch: Some("feature".to_string()), bumped_commit_hash: Some("def456".to_string()),
            last_timestamp: Some(1710547200),
            ..Default::default()
        },
        zerv_calver_tier_3()
    )]
    fn test_preset_schemas(
        #[case] schema_name: &str,
        #[case] vars: ZervVars,
        #[case] expected_schema: ZervSchema,
    ) {
        let zerv = create_zerv_version(vars, Some(schema_name), None).unwrap();
        assert_eq!(zerv.schema, expected_schema);
    }

    #[test]
    fn test_default_schema() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let zerv = create_zerv_version(vars, None, None).unwrap();
        assert_eq!(zerv.schema, zerv_standard_tier_1());
    }

    #[test]
    fn test_custom_ron_schema() {
        let vars = ZervVars::default();
        let ron_schema = r#"
            SchemaConfig(
                core: [
                    VarField(field: "major"),
                    VarField(field: "minor"),
                ],
                extra_core: [],
                build: [String(value: "custom")]
            )
        "#;

        let zerv = create_zerv_version(vars, None, Some(ron_schema)).unwrap();
        assert_eq!(zerv.schema.core.len(), 2);
        assert_eq!(zerv.schema.build.len(), 1);
    }

    #[test]
    fn test_conflicting_schemas_error() {
        let vars = ZervVars::default();
        let ron_schema = "SchemaConfig(core: [], extra_core: [], build: [])";
        let result = create_zerv_version(vars, Some("zerv-standard"), Some(ron_schema));
        assert!(matches!(result, Err(ZervError::ConflictingSchemas(_))));
    }

    #[test]
    fn test_unknown_schema_error() {
        let vars = ZervVars::default();
        let result = create_zerv_version(vars, Some("unknown"), None);
        assert!(matches!(result, Err(ZervError::UnknownSchema(_))));
    }

    #[test]
    fn test_invalid_ron_schema_error() {
        let vars = ZervVars::default();
        let invalid_ron = "invalid ron syntax";
        let result = create_zerv_version(vars, None, Some(invalid_ron));
        assert!(matches!(result, Err(ZervError::SchemaParseError(_))));
    }
}
