use std::str::FromStr;

use crate::error::ZervError;
use crate::utils::constants::timestamp_patterns;
use crate::version::zerv::{
    Component,
    Var,
    ZervSchema,
    ZervVars,
};

// Component vector definitions to reduce duplication
mod components {
    use super::*;

    pub fn standard_core() -> Vec<Component> {
        vec![
            Component::Var(Var::Major),
            Component::Var(Var::Minor),
            Component::Var(Var::Patch),
        ]
    }

    pub fn calver_core() -> Vec<Component> {
        vec![
            Component::Var(Var::Timestamp(timestamp_patterns::YYYY.to_string())),
            Component::Var(Var::Timestamp(timestamp_patterns::MM.to_string())),
            Component::Var(Var::Timestamp(timestamp_patterns::DD.to_string())),
            Component::Var(Var::Patch),
        ]
    }

    pub fn prerelease_core() -> Vec<Component> {
        vec![Component::Var(Var::Epoch), Component::Var(Var::PreRelease)]
    }

    pub fn prerelease_post_core() -> Vec<Component> {
        vec![
            Component::Var(Var::Epoch),
            Component::Var(Var::PreRelease),
            Component::Var(Var::Post),
        ]
    }

    pub fn prerelease_post_dev_core() -> Vec<Component> {
        vec![
            Component::Var(Var::Epoch),
            Component::Var(Var::PreRelease),
            Component::Var(Var::Post),
            Component::Var(Var::Dev),
        ]
    }

    pub fn build_context() -> Vec<Component> {
        vec![
            Component::Var(Var::BumpedBranch),
            Component::Var(Var::Distance),
            Component::Var(Var::BumpedCommitHashShort),
        ]
    }

    pub fn build_if_enabled(with_context: bool) -> Vec<Component> {
        if with_context {
            build_context()
        } else {
            vec![]
        }
    }

    pub fn epoch_extra_core() -> Vec<Component> {
        vec![Component::Var(Var::Epoch)]
    }
}

// Schema name constants for reuse in help text, documentation, and tests
pub mod schema_names {
    // Standard Schema Family
    pub const STANDARD: &str = "standard";
    pub const STANDARD_NO_CONTEXT: &str = "standard-no-context";
    pub const STANDARD_BASE: &str = "standard-base";
    pub const STANDARD_BASE_PRERELEASE: &str = "standard-base-prerelease";
    pub const STANDARD_BASE_PRERELEASE_POST: &str = "standard-base-prerelease-post";
    pub const STANDARD_BASE_PRERELEASE_POST_DEV: &str = "standard-base-prerelease-post-dev";
    pub const STANDARD_BASE_CONTEXT: &str = "standard-base-context";
    pub const STANDARD_BASE_PRERELEASE_CONTEXT: &str = "standard-base-prerelease-context";
    pub const STANDARD_BASE_PRERELEASE_POST_CONTEXT: &str = "standard-base-prerelease-post-context";
    pub const STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT: &str =
        "standard-base-prerelease-post-dev-context";
    pub const STANDARD_CONTEXT: &str = "standard-context";

    // CalVer Schema Family
    pub const CALVER: &str = "calver";
    pub const CALVER_NO_CONTEXT: &str = "calver-no-context";
    pub const CALVER_BASE: &str = "calver-base";
    pub const CALVER_BASE_PRERELEASE: &str = "calver-base-prerelease";
    pub const CALVER_BASE_PRERELEASE_POST: &str = "calver-base-prerelease-post";
    pub const CALVER_BASE_PRERELEASE_POST_DEV: &str = "calver-base-prerelease-post-dev";
    pub const CALVER_BASE_CONTEXT: &str = "calver-base-context";
    pub const CALVER_BASE_PRERELEASE_CONTEXT: &str = "calver-base-prerelease-context";
    pub const CALVER_BASE_PRERELEASE_POST_CONTEXT: &str = "calver-base-prerelease-post-context";
    pub const CALVER_BASE_PRERELEASE_POST_DEV_CONTEXT: &str =
        "calver-base-prerelease-post-dev-context";
    pub const CALVER_CONTEXT: &str = "calver-context";
}

/// Flexible schema variants for fine-grained version control
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionSchema {
    // Standard Schema Family (SemVer)
    Standard,
    StandardNoContext,
    StandardBase,
    StandardBasePrerelease,
    StandardBasePrereleasePost,
    StandardBasePrereleasePostDev,
    StandardBaseContext,
    StandardBasePrereleaseContext,
    StandardBasePrereleasePostContext,
    StandardBasePrereleasePostDevContext,
    StandardContext,

    // CalVer Schema Family
    Calver,
    CalverNoContext,
    CalverBase,
    CalverBasePrerelease,
    CalverBasePrereleasePost,
    CalverBasePrereleasePostDev,
    CalverBaseContext,
    CalverBasePrereleaseContext,
    CalverBasePrereleasePostContext,
    CalverBasePrereleasePostDevContext,
    CalverContext,
}

impl VersionSchema {
    /// Create a fixed schema (deterministic, no repository analysis)
    /// Used by 18 fixed schema variants that don't need ZervVars
    pub fn schema(&self) -> ZervSchema {
        match self {
            // Standard Schema Family - Fixed Variants
            VersionSchema::StandardBase => self.standard_base_schema(false),
            VersionSchema::StandardBasePrerelease => self.standard_base_prerelease_schema(false),
            VersionSchema::StandardBasePrereleasePost => {
                self.standard_base_prerelease_post_schema(false)
            }
            VersionSchema::StandardBasePrereleasePostDev => {
                self.standard_base_prerelease_post_dev_schema(false)
            }
            VersionSchema::StandardBaseContext => self.standard_base_schema(true),
            VersionSchema::StandardBasePrereleaseContext => {
                self.standard_base_prerelease_schema(true)
            }
            VersionSchema::StandardBasePrereleasePostContext => {
                self.standard_base_prerelease_post_schema(true)
            }
            VersionSchema::StandardBasePrereleasePostDevContext => {
                self.standard_base_prerelease_post_dev_schema(true)
            }

            // CalVer Schema Family - Fixed Variants
            VersionSchema::CalverBase => self.calver_base_schema(false),
            VersionSchema::CalverBasePrerelease => self.calver_base_prerelease_schema(false),
            VersionSchema::CalverBasePrereleasePost => {
                self.calver_base_prerelease_post_schema(false)
            }
            VersionSchema::CalverBasePrereleasePostDev => {
                self.calver_base_prerelease_post_dev_schema(false)
            }
            VersionSchema::CalverBaseContext => self.calver_base_schema(true),
            VersionSchema::CalverBasePrereleaseContext => self.calver_base_prerelease_schema(true),
            VersionSchema::CalverBasePrereleasePostContext => {
                self.calver_base_prerelease_post_schema(true)
            }
            VersionSchema::CalverBasePrereleasePostDevContext => {
                self.calver_base_prerelease_post_dev_schema(true)
            }

            _ => panic!(
                "Smart schemas with context (StandardContext, CalverContext) require create_schema_with_zerv()"
            ),
        }
    }

    /// Create a smart schema (analyzes repository state via ZervVars)
    /// Used by 6 smart schemas that need repository analysis for auto-detection
    pub fn schema_with_zerv(&self, vars: &ZervVars) -> ZervSchema {
        match self {
            // Standard Schema Family - Smart Variants
            VersionSchema::Standard => {
                let schema = self.smart_standard_schema(vars);
                // Add context only for dirty or distance cases (smart context)
                if vars.dirty.unwrap_or(false) || vars.distance.unwrap_or(0) > 0 {
                    schema.with_build_context()
                } else {
                    schema
                }
            }
            VersionSchema::StandardNoContext => self.smart_standard_schema(vars), // Never includes context
            VersionSchema::StandardContext => self.smart_standard_schema(vars).with_build_context(), // Always includes context

            // CalVer Schema Family - Smart Variants
            VersionSchema::Calver => {
                let schema = self.smart_calver_schema(vars);
                // Add context only for dirty or distance cases (smart context)
                if vars.dirty.unwrap_or(false) || vars.distance.unwrap_or(0) > 0 {
                    schema.with_build_context()
                } else {
                    schema
                }
            }
            VersionSchema::CalverNoContext => self.smart_calver_schema(vars), // Never includes context
            VersionSchema::CalverContext => self.smart_calver_schema(vars).with_build_context(), // Always includes context

            // Fixed schemas - delegate to schema for convenience
            fixed_schema => fixed_schema.schema(),
        }
    }

    // Helper methods for creating specific schema configurations
    fn smart_standard_schema(&self, vars: &ZervVars) -> ZervSchema {
        if vars.dirty.unwrap_or(false) {
            // Dirty => standard_base_prerelease_post_dev_schema (context added later if needed)
            self.standard_base_prerelease_post_dev_schema(false)
        } else if vars.distance.unwrap_or(0) > 0 {
            // Distance => standard_base_prerelease_post_schema (context added later if needed)
            self.standard_base_prerelease_post_schema(false)
        } else if vars.pre_release.is_some() && vars.post.is_some() {
            // Clean tagged with prerelease and post => standard_base_prerelease_post_schema without context
            self.standard_base_prerelease_post_schema(false)
        } else if vars.pre_release.is_some() {
            // Clean tagged with prerelease only => standard_base_prerelease_schema without context
            self.standard_base_prerelease_schema(false)
        } else {
            // Clean tagged (base only) => standard_base_schema without context
            self.standard_base_schema(false)
        }
    }

    fn smart_calver_schema(&self, vars: &ZervVars) -> ZervSchema {
        if vars.dirty.unwrap_or(false) {
            // Dirty => calver_base_prerelease_post_dev_schema (context added later if needed)
            self.calver_base_prerelease_post_dev_schema(false)
        } else if vars.distance.unwrap_or(0) > 0 {
            // Distance => calver_base_prerelease_post_schema (context added later if needed)
            self.calver_base_prerelease_post_schema(false)
        } else if vars.pre_release.is_some() && vars.post.is_some() {
            // Clean tagged with prerelease and post => calver_base_prerelease_post_schema without context
            self.calver_base_prerelease_post_schema(false)
        } else if vars.pre_release.is_some() {
            // Clean tagged with prerelease only => calver_base_prerelease_schema without context
            self.calver_base_prerelease_schema(false)
        } else {
            // Clean tagged (base only) => calver_base_schema without context
            self.calver_base_schema(false)
        }
    }

    fn standard_base_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::standard_core(),
            components::epoch_extra_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::standard_core(),
            components::prerelease_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_post_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::standard_core(),
            components::prerelease_post_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_post_dev_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::standard_core(),
            components::prerelease_post_dev_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::calver_core(),
            components::epoch_extra_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::calver_core(),
            components::prerelease_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_post_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::calver_core(),
            components::prerelease_post_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_post_dev_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            components::calver_core(),
            components::prerelease_post_dev_core(),
            components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }
}

impl FromStr for VersionSchema {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::schema::flexible::schema_names::*;

        match s {
            // Standard Schema Family
            STANDARD => Ok(VersionSchema::Standard),
            STANDARD_NO_CONTEXT => Ok(VersionSchema::StandardNoContext),
            STANDARD_BASE => Ok(VersionSchema::StandardBase),
            STANDARD_BASE_PRERELEASE => Ok(VersionSchema::StandardBasePrerelease),
            STANDARD_BASE_PRERELEASE_POST => Ok(VersionSchema::StandardBasePrereleasePost),
            STANDARD_BASE_PRERELEASE_POST_DEV => Ok(VersionSchema::StandardBasePrereleasePostDev),
            STANDARD_BASE_CONTEXT => Ok(VersionSchema::StandardBaseContext),
            STANDARD_BASE_PRERELEASE_CONTEXT => Ok(VersionSchema::StandardBasePrereleaseContext),
            STANDARD_BASE_PRERELEASE_POST_CONTEXT => {
                Ok(VersionSchema::StandardBasePrereleasePostContext)
            }
            STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT => {
                Ok(VersionSchema::StandardBasePrereleasePostDevContext)
            }
            STANDARD_CONTEXT => Ok(VersionSchema::StandardContext),

            // CalVer Schema Family
            CALVER => Ok(VersionSchema::Calver),
            CALVER_NO_CONTEXT => Ok(VersionSchema::CalverNoContext),
            CALVER_BASE => Ok(VersionSchema::CalverBase),
            CALVER_BASE_PRERELEASE => Ok(VersionSchema::CalverBasePrerelease),
            CALVER_BASE_PRERELEASE_POST => Ok(VersionSchema::CalverBasePrereleasePost),
            CALVER_BASE_PRERELEASE_POST_DEV => Ok(VersionSchema::CalverBasePrereleasePostDev),
            CALVER_BASE_CONTEXT => Ok(VersionSchema::CalverBaseContext),
            CALVER_BASE_PRERELEASE_CONTEXT => Ok(VersionSchema::CalverBasePrereleaseContext),
            CALVER_BASE_PRERELEASE_POST_CONTEXT => {
                Ok(VersionSchema::CalverBasePrereleasePostContext)
            }
            CALVER_BASE_PRERELEASE_POST_DEV_CONTEXT => {
                Ok(VersionSchema::CalverBasePrereleasePostDevContext)
            }
            CALVER_CONTEXT => Ok(VersionSchema::CalverContext),

            _ => Err(ZervError::UnknownSchema(s.to_string())),
        }
    }
}

/// Extension trait for ZervSchema to add context support
pub trait SchemaContextExt {
    fn with_build_context(self) -> Self;
}

impl SchemaContextExt for ZervSchema {
    fn with_build_context(self) -> Self {
        // If build context is already present, return as-is
        if !self.build().is_empty() {
            return self;
        }

        // Create new schema with build context
        ZervSchema::new_with_precedence(
            self.core().clone(),
            self.extra_core().clone(),
            components::build_context(),
            self.precedence_order().clone(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::ZervVars;

    #[test]
    fn test_version_schema_parsing() {
        use crate::schema::flexible::schema_names::*;

        // Standard schemas
        assert_eq!(
            STANDARD.parse::<VersionSchema>().unwrap(),
            VersionSchema::Standard
        );
        assert_eq!(
            STANDARD_BASE.parse::<VersionSchema>().unwrap(),
            VersionSchema::StandardBase
        );
        assert_eq!(
            STANDARD_BASE_PRERELEASE.parse::<VersionSchema>().unwrap(),
            VersionSchema::StandardBasePrerelease
        );

        // CalVer schemas
        assert_eq!(
            CALVER.parse::<VersionSchema>().unwrap(),
            VersionSchema::Calver
        );
        assert_eq!(
            CALVER_BASE.parse::<VersionSchema>().unwrap(),
            VersionSchema::CalverBase
        );
        assert_eq!(
            CALVER_BASE_PRERELEASE.parse::<VersionSchema>().unwrap(),
            VersionSchema::CalverBasePrerelease
        );
    }

    #[test]
    fn test_smart_schema_detection() {
        let clean_vars = ZervVars {
            dirty: Some(false),
            distance: Some(0),
            ..Default::default()
        };
        let distance_vars = ZervVars {
            dirty: Some(false),
            distance: Some(5),
            ..Default::default()
        };
        let dirty_vars = ZervVars {
            dirty: Some(true),
            distance: Some(0),
            ..Default::default()
        };

        let schema = VersionSchema::Standard;

        // Clean should use prerelease schema
        let clean_schema = schema.schema_with_zerv(&clean_vars);
        // Distance should use prerelease-post schema
        let _distance_schema = schema.schema_with_zerv(&distance_vars);
        // Dirty should use prerelease-post-dev schema
        let dirty_schema = schema.schema_with_zerv(&dirty_vars);

        // These should have different components
        assert_ne!(clean_schema.extra_core(), dirty_schema.extra_core());
    }

    #[test]
    fn test_all_standard_schema_variants() {
        use crate::schema::flexible::schema_names::*;

        let vars = ZervVars::default();

        // Test all standard schema variants parse correctly
        let schemas = [
            STANDARD,
            STANDARD_BASE,
            STANDARD_BASE_PRERELEASE,
            STANDARD_BASE_PRERELEASE_POST,
            STANDARD_BASE_PRERELEASE_POST_DEV,
            STANDARD_BASE_CONTEXT,
            STANDARD_BASE_PRERELEASE_CONTEXT,
            STANDARD_BASE_PRERELEASE_POST_CONTEXT,
            STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
            STANDARD_CONTEXT,
        ];

        for schema_name in schemas.iter() {
            let schema = schema_name.parse::<VersionSchema>();
            assert!(schema.is_ok(), "Failed to parse schema: {}", schema_name);

            let zerv_schema = schema.unwrap().schema_with_zerv(&vars);
            assert!(
                !zerv_schema.core().is_empty(),
                "Schema {} should have core components",
                schema_name
            );
        }
    }

    #[test]
    fn test_all_calver_schema_variants() {
        use crate::schema::flexible::schema_names::*;

        let vars = ZervVars::default();

        // Test all calver schema variants parse correctly
        let schemas = [
            CALVER,
            CALVER_BASE,
            CALVER_BASE_PRERELEASE,
            CALVER_BASE_PRERELEASE_POST,
            CALVER_BASE_PRERELEASE_POST_DEV,
            CALVER_BASE_CONTEXT,
            CALVER_BASE_PRERELEASE_CONTEXT,
            CALVER_BASE_PRERELEASE_POST_CONTEXT,
            CALVER_BASE_PRERELEASE_POST_DEV_CONTEXT,
            CALVER_CONTEXT,
        ];

        for schema_name in schemas.iter() {
            let schema = schema_name.parse::<VersionSchema>();
            assert!(schema.is_ok(), "Failed to parse schema: {}", schema_name);

            let zerv_schema = schema.unwrap().schema_with_zerv(&vars);
            assert!(
                !zerv_schema.core().is_empty(),
                "Schema {} should have core components",
                schema_name
            );
        }
    }

    #[test]
    fn test_context_vs_non_context_schemas() {
        use crate::schema::flexible::schema_names::*;

        // Test that context schemas include build context
        let base_schema = STANDARD_BASE.parse::<VersionSchema>().unwrap().schema();
        let base_context_schema = STANDARD_BASE_CONTEXT
            .parse::<VersionSchema>()
            .unwrap()
            .schema();

        assert!(
            base_context_schema.build().len() > base_schema.build().len(),
            "Context schema should have more build components than base schema"
        );

        // Test same for calver
        let calver_base_schema = CALVER_BASE.parse::<VersionSchema>().unwrap().schema();
        let calver_base_context_schema = CALVER_BASE_CONTEXT
            .parse::<VersionSchema>()
            .unwrap()
            .schema();

        assert!(
            calver_base_context_schema.build().len() > calver_base_schema.build().len(),
            "Context schema should have more build components than base schema"
        );
    }
}
