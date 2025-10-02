use crate::error::ZervError;
use crate::version::zerv::schema::ZervSchema;
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

        Ok(Self { schema, vars })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::ron_fields;
    use crate::version::zerv::Component;

    mod construction {
        use super::*;

        #[test]
        fn test_zerv_new() {
            let schema = ZervSchema {
                core: vec![Component::VarField(ron_fields::MAJOR.to_string())],
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
                    Component::VarField(ron_fields::MAJOR.to_string()),
                    Component::VarField(ron_fields::MINOR.to_string()),
                    Component::VarField(ron_fields::PATCH.to_string()),
                ],
                extra_core: vec![Component::VarField(ron_fields::PRE_RELEASE.to_string())],
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
                    Component::VarField(ron_fields::PATCH.to_string()),
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
