use crate::constants::ron_fields;
use crate::error::ZervError;
use crate::version::zerv::schema::{Component, ZervSchema};
use crate::version::zerv::vars::ZervVars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zerv {
    pub schema: ZervSchema,
    pub vars: ZervVars,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreReleaseVar {
    pub label: PreReleaseLabel,
    pub number: Option<u64>,
}

impl Zerv {
    /// Create a new Zerv object with validation
    pub fn new(schema: ZervSchema, vars: ZervVars) -> Result<Self, ZervError> {
        // Validate schema structure first
        schema.validate()?;

        // Validate schema-vars compatibility
        Self::validate_components(&schema.core, &vars)?;
        Self::validate_components(&schema.extra_core, &vars)?;
        Self::validate_components(&schema.build, &vars)?;

        Ok(Self { schema, vars })
    }

    /// Validate the Zerv object structure and compatibility
    pub fn validate(&self) -> Result<(), ZervError> {
        // Validate schema structure
        self.schema.validate()?;

        // Validate schema-vars compatibility
        Self::validate_components(&self.schema.core, &self.vars)?;
        Self::validate_components(&self.schema.extra_core, &self.vars)?;
        Self::validate_components(&self.schema.build, &self.vars)?;

        Ok(())
    }

    fn validate_components(components: &[Component], vars: &ZervVars) -> Result<(), ZervError> {
        for component in components {
            Self::validate_component(component, vars)?;
        }
        Ok(())
    }

    /// Validate a single component's compatibility with vars
    fn validate_component(component: &Component, vars: &ZervVars) -> Result<(), ZervError> {
        if let Component::VarField(field_name) = component {
            Self::check_var_field_compatibility(field_name, vars)?;
        }
        Ok(())
    }

    /// Check if a specific field has a corresponding value in vars
    fn check_var_field_compatibility(field_name: &str, vars: &ZervVars) -> Result<(), ZervError> {
        let is_missing = match field_name {
            // Core version fields
            ron_fields::MAJOR => vars.major.is_none(),
            ron_fields::MINOR => vars.minor.is_none(),
            ron_fields::PATCH => vars.patch.is_none(),
            ron_fields::EPOCH => vars.epoch.is_none(),
            // Pre-release fields
            ron_fields::PRE_RELEASE => vars.pre_release.is_none(),
            ron_fields::POST => vars.post.is_none(),
            ron_fields::DEV => vars.dev.is_none(),
            // VCS state fields
            ron_fields::DISTANCE => vars.distance.is_none(),
            ron_fields::DIRTY => vars.dirty.is_none(),
            ron_fields::BRANCH => vars.bumped_branch.is_none(),
            ron_fields::COMMIT_HASH_SHORT => vars.bumped_commit_hash.is_none(),
            // Last version fields
            ron_fields::LAST_COMMIT_HASH => vars.last_commit_hash.is_none(),
            ron_fields::LAST_TIMESTAMP => vars.last_timestamp.is_none(),
            ron_fields::LAST_BRANCH => vars.last_branch.is_none(),
            _ => {
                // Custom fields and other valid fields don't need to be present in vars
                return Ok(());
            }
        };

        if is_missing {
            // Only error for core version components (major, minor, patch)
            if matches!(
                field_name,
                ron_fields::MAJOR | ron_fields::MINOR | ron_fields::PATCH
            ) {
                return Err(ZervError::StdinError(format!(
                    "Invalid Zerv RON: schema references missing core variable '{field_name}'. Ensure this field is present in the vars section."
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod construction {
        use super::*;

        #[test]
        fn test_zerv_new() {
            let schema = ZervSchema {
                core: vec![Component::VarField("major".to_string())],
                extra_core: vec![],
                build: vec![],
            };
            let vars = ZervVars {
                major: Some(1), // Add required field for validation
                ..Default::default()
            };
            let zerv = Zerv::new(schema.clone(), vars.clone()).unwrap();

            assert_eq!(zerv.schema, schema);
            assert_eq!(zerv.vars, vars);
        }

        #[test]
        fn test_zerv_vars_default() {
            let vars = ZervVars::default();

            assert_eq!(vars.major, None);
            assert_eq!(vars.minor, None);
            assert_eq!(vars.patch, None);
            assert!(vars.custom.as_object().is_none_or(|obj| obj.is_empty()));
        }

        #[test]
        fn test_pre_release_var() {
            let pre_release = PreReleaseVar {
                label: PreReleaseLabel::Alpha,
                number: Some(1),
            };

            assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
            assert_eq!(pre_release.number, Some(1));
        }
    }

    // Validation tests using rstest

    #[rstest]
    #[case("major", true, false)] // Has value, should not error
    #[case("minor", false, true)] // Missing value, should error
    #[case("patch", false, true)] // Missing value, should error
    #[case("epoch", false, false)] // Missing value, but not core field, should not error
    #[case("pre_release", false, false)] // Missing value, but not core field, should not error
    #[case("custom.build_id", false, false)] // Custom field, should not error
    fn test_validate_component_core_fields(
        #[case] field_name: &str,
        #[case] has_value: bool,
        #[case] should_error: bool,
    ) {
        let mut vars = ZervVars::default();
        if has_value {
            match field_name {
                "major" => vars.major = Some(1),
                "minor" => vars.minor = Some(2),
                "patch" => vars.patch = Some(3),
                "epoch" => vars.epoch = Some(0),
                "pre_release" => {
                    vars.pre_release = Some(PreReleaseVar {
                        label: PreReleaseLabel::Alpha,
                        number: Some(1),
                    })
                }
                _ => {}
            }
        }

        let component = Component::VarField(field_name.to_string());
        let result = Zerv::validate_component(&component, &vars);

        if should_error {
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("missing core variable")
            );
        } else {
            assert!(result.is_ok());
        }
    }

    #[rstest]
    #[case(Component::String("test".to_string()))]
    #[case(Component::Integer(42))]
    #[case(Component::VarTimestamp("YYYY".to_string()))]
    fn test_validate_component_non_var_fields(#[case] component: Component) {
        let vars = ZervVars::default();
        let result = Zerv::validate_component(&component, &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_success() {
        let schema = ZervSchema {
            core: vec![Component::VarField("major".to_string())],
            extra_core: vec![],
            build: vec![],
        };
        let vars = ZervVars {
            major: Some(1),
            ..Default::default()
        };

        let zerv = Zerv { schema, vars };
        assert!(zerv.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_core_field() {
        let schema = ZervSchema {
            core: vec![Component::VarField("major".to_string())],
            extra_core: vec![],
            build: vec![],
        };
        let vars = ZervVars::default(); // No major field

        let zerv = Zerv { schema, vars };
        let result = zerv.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing core variable")
        );
    }

    #[test]
    fn test_validate_components_success() {
        let schema = ZervSchema {
            core: vec![
                Component::VarField("major".to_string()),
                Component::VarField("minor".to_string()),
            ],
            extra_core: vec![],
            build: vec![],
        };
        let vars = ZervVars {
            major: Some(1),
            minor: Some(2),
            ..Default::default()
        };

        let result = Zerv::validate_components(&schema.core, &vars);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_components_missing_field() {
        let schema = ZervSchema {
            core: vec![Component::VarField("major".to_string())],
            extra_core: vec![],
            build: vec![],
        };
        let vars = ZervVars::default(); // No major field

        let result = Zerv::validate_components(&schema.core, &vars);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing core variable")
        );
    }

    // Component tests moved to schema module

    mod variables {
        use super::*;

        #[test]
        fn test_zerv_vars_with_values() {
            let vars = ZervVars {
                major: Some(1),
                minor: Some(2),
                patch: Some(3),
                bumped_branch: Some("main".to_string()),
                dirty: Some(true),
                ..Default::default()
            };

            assert_eq!(vars.major, Some(1));
            assert_eq!(vars.minor, Some(2));
            assert_eq!(vars.patch, Some(3));
            assert_eq!(vars.bumped_branch, Some("main".to_string()));
            assert_eq!(vars.dirty, Some(true));
        }

        #[test]
        fn test_custom_variables() {
            let vars = ZervVars {
                custom: serde_json::json!({
                    "build_id": 456,
                    "env": "prod",
                    "debug": false,
                    "metadata": {
                        "author": "ci",
                        "timestamp": 1703123456
                    }
                }),
                ..Default::default()
            };

            assert_eq!(vars.custom["build_id"], serde_json::json!(456));
            assert_eq!(vars.custom["env"], serde_json::json!("prod"));
            assert_eq!(vars.custom["debug"], serde_json::json!(false));
            assert_eq!(vars.custom["metadata"]["author"], serde_json::json!("ci"));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_schema() {
            let schema = ZervSchema {
                core: vec![],
                extra_core: vec![],
                build: vec![],
            };
            let vars = ZervVars::default();
            // Empty schema should fail validation
            let result = Zerv::new(schema, vars);
            assert!(result.is_err());
        }

        #[test]
        fn test_pre_release_without_number() {
            let pre_release = PreReleaseVar {
                label: PreReleaseLabel::Alpha,
                number: None,
            };

            assert_eq!(pre_release.label, PreReleaseLabel::Alpha);
            assert_eq!(pre_release.number, None);
        }

        #[test]
        fn test_max_values() {
            let vars = ZervVars {
                major: Some(u64::MAX),
                distance: Some(u64::MAX),
                ..Default::default()
            };

            assert_eq!(vars.major, Some(u64::MAX));
            assert_eq!(vars.distance, Some(u64::MAX));
        }

        #[test]
        fn test_empty_strings() {
            let vars = ZervVars {
                bumped_branch: Some("".to_string()),
                last_commit_hash: Some("".to_string()),
                ..Default::default()
            };

            assert_eq!(vars.bumped_branch, Some("".to_string()));
            assert_eq!(vars.last_commit_hash, Some("".to_string()));
        }
    }

    mod complex_structures {
        use super::*;

        #[test]
        fn test_semver_like_structure() {
            let schema = ZervSchema {
                core: vec![
                    Component::VarField("major".to_string()),
                    Component::VarField("minor".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core: vec![Component::VarField("pre_release".to_string())],
                build: vec![
                    Component::String("build".to_string()),
                    Component::Integer(123),
                ],
            };

            let vars = ZervVars {
                major: Some(1),
                minor: Some(2),
                patch: Some(3),
                pre_release: Some(PreReleaseVar {
                    label: PreReleaseLabel::Alpha,
                    number: Some(1),
                }),
                ..Default::default()
            };

            let zerv = Zerv::new(schema, vars).unwrap();
            assert_eq!(zerv.vars.major, Some(1));
            assert_eq!(
                zerv.vars
                    .pre_release
                    .as_ref()
                    .expect("Test setup error: pre_release should be Some")
                    .label,
                PreReleaseLabel::Alpha
            );
        }

        #[test]
        fn test_calver_like_structure() {
            let schema = ZervSchema {
                core: vec![
                    Component::VarTimestamp("YYYY".to_string()),
                    Component::VarTimestamp("MM".to_string()),
                    Component::VarTimestamp("DD".to_string()),
                    Component::VarField("patch".to_string()),
                ],
                extra_core: vec![],
                build: vec![],
            };

            let vars = ZervVars {
                patch: Some(1),
                last_timestamp: Some(1710547200),
                ..Default::default()
            };

            let zerv = Zerv::new(schema, vars).unwrap();
            assert_eq!(zerv.vars.patch, Some(1));
            assert_eq!(zerv.vars.last_timestamp, Some(1710547200));
        }
    }
}
