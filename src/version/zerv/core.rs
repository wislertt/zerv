use serde::{Deserialize, Serialize};
use serde_json;

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
pub struct ZervSchema {
    pub core: Vec<Component>,
    pub extra_core: Vec<Component>,
    pub build: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Component {
    #[serde(rename = "str")]
    String(String),
    #[serde(rename = "int")]
    Integer(u64),
    #[serde(rename = "var")]
    VarField(String),
    #[serde(rename = "ts")]
    VarTimestamp(String),
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ZervVars {
    // Core version fields (unchanged)
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub epoch: Option<u64>,
    pub pre_release: Option<PreReleaseVar>,
    pub post: Option<u64>,
    pub dev: Option<u64>,

    // VCS state fields (renamed and restructured)
    pub distance: Option<u64>,
    pub dirty: Option<bool>,

    // Bumped fields (for template access)
    pub bumped_branch: Option<String>,
    pub bumped_commit_hash: Option<String>,
    pub bumped_commit_hash_short: Option<String>,
    pub bumped_timestamp: Option<u64>,

    // Last version fields (for template access)
    pub last_branch: Option<String>,
    pub last_commit_hash: Option<String>,
    pub last_timestamp: Option<u64>,

    // Custom variables (changed to nested JSON)
    #[serde(default = "default_custom_value")]
    pub custom: serde_json::Value,
}

/// Default value for custom field - returns an empty JSON object
fn default_custom_value() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreReleaseVar {
    pub label: PreReleaseLabel,
    pub number: Option<u64>,
}

impl Zerv {
    pub fn new(schema: ZervSchema, vars: ZervVars) -> Self {
        Self { schema, vars }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod construction {
        use super::*;

        #[test]
        fn test_zerv_new() {
            let schema = ZervSchema {
                core: vec![Component::VarField("major".to_string())],
                extra_core: vec![],
                build: vec![],
            };
            let vars = ZervVars::default();
            let zerv = Zerv::new(schema.clone(), vars.clone());

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

    mod components {
        use super::*;

        #[test]
        fn test_component_string() {
            let comp = Component::String("test".to_string());
            match comp {
                Component::String(s) => assert_eq!(s, "test"),
                _ => panic!("Expected String component"),
            }
        }

        #[test]
        fn test_component_integer() {
            let comp = Component::Integer(42);
            match comp {
                Component::Integer(n) => assert_eq!(n, 42),
                _ => panic!("Expected Integer component"),
            }
        }

        #[test]
        fn test_component_var_field() {
            let comp = Component::VarField("major".to_string());
            match comp {
                Component::VarField(field) => assert_eq!(field, "major"),
                _ => panic!("Expected VarField component"),
            }
        }

        #[test]
        fn test_component_var_timestamp() {
            let comp = Component::VarTimestamp("YYYY".to_string());
            match comp {
                Component::VarTimestamp(pattern) => assert_eq!(pattern, "YYYY"),
                _ => panic!("Expected VarTimestamp component"),
            }
        }

        // VarCustom component removed - use var("custom.xxx") instead
    }

    // VarValue tests removed - using serde_json::Value for custom variables

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
            let zerv = Zerv::new(schema, vars);

            assert!(zerv.schema.core.is_empty());
            assert!(zerv.schema.extra_core.is_empty());
            assert!(zerv.schema.build.is_empty());
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

            let zerv = Zerv::new(schema, vars);
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

            let zerv = Zerv::new(schema, vars);
            assert_eq!(zerv.vars.patch, Some(1));
            assert_eq!(zerv.vars.last_timestamp, Some(1710547200));
        }
    }
}
