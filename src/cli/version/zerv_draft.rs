use crate::cli::version::args::VersionArgs;
use crate::error::ZervError;
use crate::schema::{
    get_preset_schema,
    parse_ron_schema,
};
use crate::version::zerv::{
    Zerv,
    ZervSchema,
    ZervVars,
};

/// Intermediate structure for version processing before final Zerv creation
/// Contains ZervVars and optional schema (Some for stdin, None for git)
#[derive(Debug, Clone)]
pub struct ZervDraft {
    pub vars: ZervVars,
    pub schema: Option<ZervSchema>, // Some for stdin, None for git
}

impl ZervDraft {
    pub fn new(vars: ZervVars, schema: Option<ZervSchema>) -> Self {
        Self { vars, schema }
    }

    pub fn to_zerv(mut self, args: &VersionArgs) -> Result<Zerv, ZervError> {
        // Apply overrides first
        self.vars.apply_context_overrides(args)?;

        // Then create the Zerv object
        let (schema_name, schema_ron) = args.resolve_schema();
        let mut zerv = self.create_zerv_version(schema_name, schema_ron)?;

        // Apply component processing (bumps with reset logic)
        zerv.apply_component_processing(args)?;
        zerv.normalize();

        Ok(zerv)
    }

    pub fn create_zerv_version(
        self,
        schema_name: Option<&str>,
        schema_ron: Option<&str>,
    ) -> Result<Zerv, ZervError> {
        // Move the logic from crate::schema::create_zerv_version here
        let schema = match (schema_name, schema_ron) {
            // Custom RON schema
            (None, Some(ron_str)) => parse_ron_schema(ron_str)?,

            // Built-in schema
            (Some(name), None) => {
                if let Some(schema) = get_preset_schema(name, &self.vars) {
                    schema
                } else {
                    return Err(ZervError::UnknownSchema(name.to_string()));
                }
            }

            // Error cases
            (Some(_), Some(_)) => {
                return Err(ZervError::ConflictingSchemas(
                    "Cannot specify both schema_name and schema_ron".to_string(),
                ));
            }
            (None, None) => {
                // If no new schema requested, use existing schema from stdin source
                if let Some(existing_schema) = self.schema {
                    existing_schema
                } else {
                    return Err(ZervError::MissingSchema(
                        "Either schema_name or schema_ron must be provided".to_string(),
                    ));
                }
            }
        };

        Zerv::new(schema, self.vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::zerv_standard_tier_1;

    #[test]
    fn test_zerv_draft_creation() {
        let vars = ZervVars::default();
        let draft = ZervDraft::new(vars.clone(), None);
        assert_eq!(draft.vars, vars);
        assert!(draft.schema.is_none());

        use crate::version::zerv::bump::precedence::PrecedenceOrder;
        use crate::version::zerv::{
            Component,
            Var,
        };
        let schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major)],
            vec![],
            vec![],
            PrecedenceOrder::default(),
        )
        .unwrap();
        let draft_with_schema = ZervDraft::new(vars, Some(schema.clone()));
        assert_eq!(draft_with_schema.schema, Some(schema));
    }

    #[test]
    fn test_to_zerv_with_overrides() {
        use crate::cli::version::args::{
            OverridesConfig,
            VersionArgs,
        };

        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            ..Default::default()
        };

        let args = VersionArgs {
            overrides: OverridesConfig {
                tag_version: Some("5.0.0".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let draft = ZervDraft::new(vars, None);
        let zerv = draft.to_zerv(&args).unwrap();
        assert_eq!(zerv.vars.major, Some(5));
        assert_eq!(zerv.vars.minor, Some(0));
        assert_eq!(zerv.vars.patch, Some(0));
    }

    #[test]
    fn test_create_zerv_version_with_preset_schema() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        // Test that create_zerv_version requires explicit schema (no default)
        let draft = ZervDraft::new(vars.clone(), None);
        let result = draft.create_zerv_version(None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZervError::MissingSchema(_)));

        // Test with explicit schema (should work)
        let draft = ZervDraft::new(vars, None);
        let zerv = draft
            .create_zerv_version(Some("zerv-standard"), None)
            .unwrap();
        assert_eq!(zerv.schema, zerv_standard_tier_1());
    }

    #[test]
    fn test_custom_ron_schema() {
        let vars = ZervVars::default();
        let ron_schema = r#"
            ZervSchema(
                core: [
                    var(Major),
                    var(Minor),
                ],
                extra_core: [],
                build: [str("custom")],
                precedence_order: []
            )
        "#;

        let draft = ZervDraft::new(vars, None);
        let zerv = draft.create_zerv_version(None, Some(ron_schema)).unwrap();
        assert_eq!(zerv.schema.core().len(), 2);
        assert_eq!(zerv.schema.build().len(), 1);
    }

    #[test]
    fn test_conflicting_schemas_error() {
        let vars = ZervVars::default();
        let ron_schema = "ZervSchema(core: [], extra_core: [], build: [], precedence_order: [])";
        let draft = ZervDraft::new(vars, None);
        let result = draft.create_zerv_version(Some("zerv-standard"), Some(ron_schema));
        assert!(matches!(result, Err(ZervError::ConflictingSchemas(_))));
    }

    #[test]
    fn test_unknown_schema_error() {
        let vars = ZervVars::default();
        let draft = ZervDraft::new(vars, None);
        let result = draft.create_zerv_version(Some("unknown"), None);
        assert!(matches!(result, Err(ZervError::UnknownSchema(_))));
    }

    #[test]
    fn test_invalid_ron_schema_error() {
        let vars = ZervVars::default();
        let invalid_ron = "invalid ron syntax";
        let draft = ZervDraft::new(vars, None);
        let result = draft.create_zerv_version(None, Some(invalid_ron));
        assert!(matches!(result, Err(ZervError::StdinError(_))));
    }

    #[test]
    fn test_use_existing_schema_from_stdin() {
        use crate::version::zerv::bump::precedence::PrecedenceOrder;
        use crate::version::zerv::{
            Component,
            Var,
        };

        let vars = ZervVars::default();
        let existing_schema = ZervSchema::new_with_precedence(
            vec![Component::Var(Var::Major)],
            vec![],
            vec![],
            PrecedenceOrder::default(),
        )
        .unwrap();

        // Test using existing schema when no new schema is provided
        let draft = ZervDraft::new(vars, Some(existing_schema));
        let zerv = draft.create_zerv_version(None, None).unwrap();
        assert_eq!(zerv.schema.core().len(), 1);
        assert_eq!(zerv.schema.extra_core().len(), 0);
        assert_eq!(zerv.schema.build().len(), 0);
    }

    #[test]
    fn test_zerv_schema_structure() {
        use crate::version::zerv::{
            Component,
            Var,
        };

        // Create a simple ZervVars for tier 1 (tagged, clean)
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let draft = ZervDraft::new(vars, None);
        let zerv = draft
            .create_zerv_version(Some("zerv-standard"), None)
            .unwrap();

        // Test the actual schema structure
        println!("Core components: {:?}", zerv.schema.core());
        println!("Extra core components: {:?}", zerv.schema.extra_core());
        println!("Build components: {:?}", zerv.schema.build());

        // Verify core structure
        assert_eq!(zerv.schema.core().len(), 3);
        assert_eq!(zerv.schema.core()[0], Component::Var(Var::Major));
        assert_eq!(zerv.schema.core()[1], Component::Var(Var::Minor));
        assert_eq!(zerv.schema.core()[2], Component::Var(Var::Patch));
    }

    #[test]
    fn test_zerv_ron_roundtrip_schema() {
        use crate::version::zerv::{
            Component,
            Var,
        };

        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let draft = ZervDraft::new(vars, None);
        let original = draft
            .create_zerv_version(Some("zerv-standard"), None)
            .unwrap();
        let ron_string = original.to_string();
        let parsed: Zerv = ron_string.parse().unwrap();

        // Verify schema is preserved
        assert_eq!(parsed.schema.core().len(), 3);
        assert_eq!(parsed.schema.core()[0], Component::Var(Var::Major));
        assert_eq!(parsed.schema.core()[1], Component::Var(Var::Minor));
        assert_eq!(parsed.schema.core()[2], Component::Var(Var::Patch));

        // Verify vars are preserved
        assert_eq!(parsed.vars.major, Some(1));
        assert_eq!(parsed.vars.minor, Some(2));
        assert_eq!(parsed.vars.patch, Some(3));
    }
}
