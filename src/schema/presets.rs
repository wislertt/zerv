use std::str::FromStr;

use crate::error::ZervError;
use crate::utils::constants::timestamp_patterns;
use crate::version::zerv::{
    Component,
    Var,
    ZervSchema,
    ZervVars,
};

mod schema_preset_components {
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

pub mod schema_preset_names {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZervSchemaPreset {
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

impl ZervSchemaPreset {
    pub fn schema(&self) -> ZervSchema {
        match self {
            ZervSchemaPreset::StandardBase => self.standard_base_schema(false),
            ZervSchemaPreset::StandardBasePrerelease => self.standard_base_prerelease_schema(false),
            ZervSchemaPreset::StandardBasePrereleasePost => {
                self.standard_base_prerelease_post_schema(false)
            }
            ZervSchemaPreset::StandardBasePrereleasePostDev => {
                self.standard_base_prerelease_post_dev_schema(false)
            }
            ZervSchemaPreset::StandardBaseContext => self.standard_base_schema(true),
            ZervSchemaPreset::StandardBasePrereleaseContext => {
                self.standard_base_prerelease_schema(true)
            }
            ZervSchemaPreset::StandardBasePrereleasePostContext => {
                self.standard_base_prerelease_post_schema(true)
            }
            ZervSchemaPreset::StandardBasePrereleasePostDevContext => {
                self.standard_base_prerelease_post_dev_schema(true)
            }

            // CalVer Schema Family - Fixed Variants
            ZervSchemaPreset::CalverBase => self.calver_base_schema(false),
            ZervSchemaPreset::CalverBasePrerelease => self.calver_base_prerelease_schema(false),
            ZervSchemaPreset::CalverBasePrereleasePost => {
                self.calver_base_prerelease_post_schema(false)
            }
            ZervSchemaPreset::CalverBasePrereleasePostDev => {
                self.calver_base_prerelease_post_dev_schema(false)
            }
            ZervSchemaPreset::CalverBaseContext => self.calver_base_schema(true),
            ZervSchemaPreset::CalverBasePrereleaseContext => {
                self.calver_base_prerelease_schema(true)
            }
            ZervSchemaPreset::CalverBasePrereleasePostContext => {
                self.calver_base_prerelease_post_schema(true)
            }
            ZervSchemaPreset::CalverBasePrereleasePostDevContext => {
                self.calver_base_prerelease_post_dev_schema(true)
            }

            _ => panic!(
                "Smart schemas with context (StandardContext, CalverContext) require create_schema_with_zerv()"
            ),
        }
    }

    pub fn schema_with_zerv(&self, vars: &ZervVars) -> ZervSchema {
        match self {
            ZervSchemaPreset::Standard => {
                self.with_smart_build_context(self.smart_standard_schema(vars), vars)
            }
            ZervSchemaPreset::StandardNoContext => self.smart_standard_schema(vars),
            ZervSchemaPreset::StandardContext => {
                self.with_build_context(self.smart_standard_schema(vars))
            }

            ZervSchemaPreset::Calver => {
                self.with_smart_build_context(self.smart_calver_schema(vars), vars)
            }
            ZervSchemaPreset::CalverNoContext => self.smart_calver_schema(vars),
            ZervSchemaPreset::CalverContext => {
                self.with_build_context(self.smart_calver_schema(vars))
            }

            fixed_schema => fixed_schema.schema(),
        }
    }

    fn smart_standard_schema(&self, vars: &ZervVars) -> ZervSchema {
        if vars.dirty.unwrap_or(false) {
            self.standard_base_prerelease_post_dev_schema(false)
        } else if vars.distance.unwrap_or(0) > 0
            || (vars.pre_release.is_some() && vars.post.is_some())
        {
            self.standard_base_prerelease_post_schema(false)
        } else if vars.pre_release.is_some() {
            self.standard_base_prerelease_schema(false)
        } else {
            self.standard_base_schema(false)
        }
    }

    fn smart_calver_schema(&self, vars: &ZervVars) -> ZervSchema {
        if vars.dirty.unwrap_or(false) {
            self.calver_base_prerelease_post_dev_schema(false)
        } else if vars.distance.unwrap_or(0) > 0
            || (vars.pre_release.is_some() && vars.post.is_some())
        {
            self.calver_base_prerelease_post_schema(false)
        } else if vars.pre_release.is_some() {
            self.calver_base_prerelease_schema(false)
        } else {
            self.calver_base_schema(false)
        }
    }

    fn standard_base_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::standard_core(),
            schema_preset_components::epoch_extra_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::standard_core(),
            schema_preset_components::prerelease_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_post_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::standard_core(),
            schema_preset_components::prerelease_post_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_post_dev_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::standard_core(),
            schema_preset_components::prerelease_post_dev_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::calver_core(),
            schema_preset_components::epoch_extra_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::calver_core(),
            schema_preset_components::prerelease_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_post_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::calver_core(),
            schema_preset_components::prerelease_post_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_post_dev_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            schema_preset_components::calver_core(),
            schema_preset_components::prerelease_post_dev_core(),
            schema_preset_components::build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn with_build_context(&self, schema: ZervSchema) -> ZervSchema {
        let mut result = schema;
        result
            .set_build(schema_preset_components::build_context())
            .unwrap();
        result
    }

    fn with_smart_build_context(&self, schema: ZervSchema, vars: &ZervVars) -> ZervSchema {
        if vars.dirty.unwrap_or(false) || vars.distance.unwrap_or(0) > 0 {
            self.with_build_context(schema)
        } else {
            schema
        }
    }
}

impl FromStr for ZervSchemaPreset {
    type Err = ZervError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::schema::presets::schema_preset_names::*;

        match s {
            // Standard Schema Family
            STANDARD => Ok(ZervSchemaPreset::Standard),
            STANDARD_NO_CONTEXT => Ok(ZervSchemaPreset::StandardNoContext),
            STANDARD_BASE => Ok(ZervSchemaPreset::StandardBase),
            STANDARD_BASE_PRERELEASE => Ok(ZervSchemaPreset::StandardBasePrerelease),
            STANDARD_BASE_PRERELEASE_POST => Ok(ZervSchemaPreset::StandardBasePrereleasePost),
            STANDARD_BASE_PRERELEASE_POST_DEV => {
                Ok(ZervSchemaPreset::StandardBasePrereleasePostDev)
            }
            STANDARD_BASE_CONTEXT => Ok(ZervSchemaPreset::StandardBaseContext),
            STANDARD_BASE_PRERELEASE_CONTEXT => Ok(ZervSchemaPreset::StandardBasePrereleaseContext),
            STANDARD_BASE_PRERELEASE_POST_CONTEXT => {
                Ok(ZervSchemaPreset::StandardBasePrereleasePostContext)
            }
            STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT => {
                Ok(ZervSchemaPreset::StandardBasePrereleasePostDevContext)
            }
            STANDARD_CONTEXT => Ok(ZervSchemaPreset::StandardContext),

            CALVER => Ok(ZervSchemaPreset::Calver),
            CALVER_NO_CONTEXT => Ok(ZervSchemaPreset::CalverNoContext),
            CALVER_BASE => Ok(ZervSchemaPreset::CalverBase),
            CALVER_BASE_PRERELEASE => Ok(ZervSchemaPreset::CalverBasePrerelease),
            CALVER_BASE_PRERELEASE_POST => Ok(ZervSchemaPreset::CalverBasePrereleasePost),
            CALVER_BASE_PRERELEASE_POST_DEV => Ok(ZervSchemaPreset::CalverBasePrereleasePostDev),
            CALVER_BASE_CONTEXT => Ok(ZervSchemaPreset::CalverBaseContext),
            CALVER_BASE_PRERELEASE_CONTEXT => Ok(ZervSchemaPreset::CalverBasePrereleaseContext),
            CALVER_BASE_PRERELEASE_POST_CONTEXT => {
                Ok(ZervSchemaPreset::CalverBasePrereleasePostContext)
            }
            CALVER_BASE_PRERELEASE_POST_DEV_CONTEXT => {
                Ok(ZervSchemaPreset::CalverBasePrereleasePostDevContext)
            }
            CALVER_CONTEXT => Ok(ZervSchemaPreset::CalverContext),

            _ => Err(ZervError::UnknownSchema(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::zerv::ZervVars;

    #[test]
    fn test_version_schema_parsing() {
        use crate::schema::presets::schema_preset_names::*;

        assert_eq!(
            STANDARD.parse::<ZervSchemaPreset>().unwrap(),
            ZervSchemaPreset::Standard
        );
        assert_eq!(
            STANDARD_BASE.parse::<ZervSchemaPreset>().unwrap(),
            ZervSchemaPreset::StandardBase
        );
        assert_eq!(
            STANDARD_BASE_PRERELEASE
                .parse::<ZervSchemaPreset>()
                .unwrap(),
            ZervSchemaPreset::StandardBasePrerelease
        );

        assert_eq!(
            CALVER.parse::<ZervSchemaPreset>().unwrap(),
            ZervSchemaPreset::Calver
        );
        assert_eq!(
            CALVER_BASE.parse::<ZervSchemaPreset>().unwrap(),
            ZervSchemaPreset::CalverBase
        );
        assert_eq!(
            CALVER_BASE_PRERELEASE.parse::<ZervSchemaPreset>().unwrap(),
            ZervSchemaPreset::CalverBasePrerelease
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

        let schema = ZervSchemaPreset::Standard;

        let clean_schema = schema.schema_with_zerv(&clean_vars);
        let _distance_schema = schema.schema_with_zerv(&distance_vars);
        let dirty_schema = schema.schema_with_zerv(&dirty_vars);

        assert_ne!(clean_schema.extra_core(), dirty_schema.extra_core());
    }

    #[test]
    fn test_all_standard_schema_variants() {
        use crate::schema::presets::schema_preset_names::*;

        let vars = ZervVars::default();

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
            let schema = schema_name.parse::<ZervSchemaPreset>();
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
        use crate::schema::presets::schema_preset_names::*;

        let vars = ZervVars::default();

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
            let schema = schema_name.parse::<ZervSchemaPreset>();
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
        use crate::schema::presets::schema_preset_names::*;

        let base_schema = STANDARD_BASE.parse::<ZervSchemaPreset>().unwrap().schema();
        let base_context_schema = STANDARD_BASE_CONTEXT
            .parse::<ZervSchemaPreset>()
            .unwrap()
            .schema();

        assert!(
            base_context_schema.build().len() > base_schema.build().len(),
            "Context schema should have more build components than base schema"
        );

        let calver_base_schema = CALVER_BASE.parse::<ZervSchemaPreset>().unwrap().schema();
        let calver_base_context_schema = CALVER_BASE_CONTEXT
            .parse::<ZervSchemaPreset>()
            .unwrap()
            .schema();

        assert!(
            calver_base_context_schema.build().len() > calver_base_schema.build().len(),
            "Context schema should have more build components than base schema"
        );
    }
}
