use std::str::FromStr;

pub use super::components::{
    build_context,
    build_if_enabled,
    calver_core,
    epoch_extra_core,
    prerelease_core,
    prerelease_post_core,
    prerelease_post_dev_core,
    standard_core,
};
pub use super::names::schema_preset_names::{
    CALVER,
    CALVER_BASE,
    CALVER_BASE_CONTEXT,
    CALVER_BASE_PRERELEASE,
    CALVER_BASE_PRERELEASE_CONTEXT,
    CALVER_BASE_PRERELEASE_POST,
    CALVER_BASE_PRERELEASE_POST_CONTEXT,
    CALVER_BASE_PRERELEASE_POST_DEV,
    CALVER_BASE_PRERELEASE_POST_DEV_CONTEXT,
    CALVER_CONTEXT,
    CALVER_NO_CONTEXT,
    STANDARD,
    STANDARD_BASE,
    STANDARD_BASE_CONTEXT,
    STANDARD_BASE_PRERELEASE,
    STANDARD_BASE_PRERELEASE_CONTEXT,
    STANDARD_BASE_PRERELEASE_POST,
    STANDARD_BASE_PRERELEASE_POST_CONTEXT,
    STANDARD_BASE_PRERELEASE_POST_DEV,
    STANDARD_BASE_PRERELEASE_POST_DEV_CONTEXT,
    STANDARD_CONTEXT,
    STANDARD_NO_CONTEXT,
};
use crate::error::ZervError;
use crate::version::zerv::{
    ZervSchema,
    ZervVars,
};

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
            standard_core(),
            epoch_extra_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            standard_core(),
            prerelease_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_post_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            standard_core(),
            prerelease_post_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn standard_base_prerelease_post_dev_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            standard_core(),
            prerelease_post_dev_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            calver_core(),
            epoch_extra_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            calver_core(),
            prerelease_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_post_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            calver_core(),
            prerelease_post_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn calver_base_prerelease_post_dev_schema(&self, with_context: bool) -> ZervSchema {
        ZervSchema::new_with_precedence(
            calver_core(),
            prerelease_post_dev_core(),
            build_if_enabled(with_context),
            Default::default(),
        )
        .unwrap()
    }

    fn with_build_context(&self, schema: ZervSchema) -> ZervSchema {
        let mut result = schema;
        result.set_build(build_context()).unwrap();
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
