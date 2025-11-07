use crate::cli::version::args::{
    ResolvedArgs,
    VersionArgs,
};
use crate::error::ZervError;
use crate::schema::{
    VersionSchema,
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
        // let (schema_name, schema_ron) = args.resolve_schema();
        let mut zerv = self.create_zerv_version(args)?;

        // Resolve templates using the current Zerv state
        let resolved_args = ResolvedArgs::resolve(args, &zerv)?;

        // Apply component processing (bumps with reset logic)
        zerv.apply_component_processing(&resolved_args)?;
        zerv.normalize();

        Ok(zerv)
    }

    fn resolve_schema(
        schema_name: Option<&str>,
        schema_ron: Option<&str>,
        existing_schema: Option<ZervSchema>,
        vars: &ZervVars,
    ) -> Result<ZervSchema, ZervError> {
        match (schema_name, schema_ron) {
            // Custom RON schema
            (None, Some(ron_str)) => parse_ron_schema(ron_str),

            // Built-in schema
            (Some(name), None) => match name.parse::<VersionSchema>() {
                Ok(schema) => Ok(schema.schema_with_zerv(vars)),
                Err(_) => Err(ZervError::UnknownSchema(name.to_string())),
            },

            // Error cases
            (Some(_), Some(_)) => Err(ZervError::ConflictingSchemas(
                "Cannot specify both schema_name and schema_ron".to_string(),
            )),
            (None, None) => {
                // If no new schema requested, use existing schema from stdin source
                if let Some(existing_schema) = existing_schema {
                    Ok(existing_schema)
                } else {
                    Ok(VersionSchema::Standard.schema_with_zerv(vars))
                }
            }
        }
    }

    pub fn create_zerv_version(self, args: &VersionArgs) -> Result<Zerv, ZervError> {
        let schema = Self::resolve_schema(
            args.main.schema.as_deref(),
            args.main.schema_ron.as_deref(),
            self.schema,
            &self.vars,
        )?;
        Zerv::new(schema, self.vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::version::args::{
        MainConfig,
        OverridesConfig,
        VersionArgs,
    };
    use crate::schema::schema_names;
    use crate::version::zerv::bump::precedence::PrecedenceOrder;
    use crate::version::zerv::{
        Component,
        Var,
    };

    #[test]
    fn test_zerv_draft_creation() {
        let vars = ZervVars::default();
        let draft = ZervDraft::new(vars.clone(), None);
        assert_eq!(draft.vars, vars);
        assert!(draft.schema.is_none());
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
        use crate::schema::VersionSchema;
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        // Test with no schema (should use default)
        let draft = ZervDraft::new(vars.clone(), None);
        let args = VersionArgs::default();
        let zerv = draft.create_zerv_version(&args).unwrap();
        assert_eq!(zerv.schema, VersionSchema::StandardBase.schema());

        // Test with explicit schema
        let draft = ZervDraft::new(vars, None);
        let args = VersionArgs {
            main: MainConfig {
                schema: Some(schema_names::STANDARD.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let zerv = draft.create_zerv_version(&args).unwrap();
        assert_eq!(zerv.schema, VersionSchema::StandardBase.schema());
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
        let args = VersionArgs {
            main: MainConfig {
                schema_ron: Some(ron_schema.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let zerv = draft.create_zerv_version(&args).unwrap();
        assert_eq!(zerv.schema.core().len(), 2);
        assert_eq!(zerv.schema.build().len(), 1);
    }

    #[test]
    fn test_conflicting_schemas_error() {
        let vars = ZervVars::default();
        let ron_schema = "ZervSchema(core: [], extra_core: [], build: [], precedence_order: [])";
        let draft = ZervDraft::new(vars, None);
        let args = VersionArgs {
            main: MainConfig {
                schema: Some(schema_names::STANDARD.to_string()),
                schema_ron: Some(ron_schema.to_string()),
            },
            ..Default::default()
        };
        let result = draft.create_zerv_version(&args);
        assert!(matches!(result, Err(ZervError::ConflictingSchemas(_))));
    }

    #[test]
    fn test_unknown_schema_error() {
        let vars = ZervVars::default();
        let draft = ZervDraft::new(vars, None);
        let args = VersionArgs {
            main: MainConfig {
                schema: Some("unknown".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = draft.create_zerv_version(&args);
        assert!(matches!(result, Err(ZervError::UnknownSchema(_))));
    }

    #[test]
    fn test_invalid_ron_schema_error() {
        let vars = ZervVars::default();
        let invalid_ron = "invalid ron syntax";
        let draft = ZervDraft::new(vars, None);
        let args = VersionArgs {
            main: MainConfig {
                schema_ron: Some(invalid_ron.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = draft.create_zerv_version(&args);
        assert!(matches!(result, Err(ZervError::StdinError(_))));
    }

    #[test]
    fn test_use_existing_schema_from_stdin() {
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
        let args = VersionArgs::default();
        let zerv = draft.create_zerv_version(&args).unwrap();
        assert_eq!(zerv.schema.core().len(), 1);
        assert_eq!(zerv.schema.extra_core().len(), 0);
        assert_eq!(zerv.schema.build().len(), 0);
    }

    #[test]
    fn test_zerv_schema_structure() {
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
        let args = VersionArgs {
            main: MainConfig {
                schema: Some(schema_names::STANDARD.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let zerv = draft.create_zerv_version(&args).unwrap();

        // Test the actual schema structure
        tracing::debug!("Core components: {:?}", zerv.schema.core());
        tracing::debug!("Extra core components: {:?}", zerv.schema.extra_core());
        tracing::debug!("Build components: {:?}", zerv.schema.build());

        // Verify core structure
        assert_eq!(zerv.schema.core().len(), 3);
        assert_eq!(zerv.schema.core()[0], Component::Var(Var::Major));
        assert_eq!(zerv.schema.core()[1], Component::Var(Var::Minor));
        assert_eq!(zerv.schema.core()[2], Component::Var(Var::Patch));
    }

    #[test]
    fn test_zerv_ron_roundtrip_schema() {
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };

        let draft = ZervDraft::new(vars, None);
        let args = VersionArgs {
            main: MainConfig {
                schema: Some(schema_names::STANDARD.to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let original = draft.create_zerv_version(&args).unwrap();
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
