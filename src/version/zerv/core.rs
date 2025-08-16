use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Zerv {
    pub format: ZervFormat,
    pub vars: ZervVars,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZervFormat {
    pub core: Vec<Component>,
    pub extra_core: Vec<Component>,
    pub build: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    String(String),
    Integer(u64),
    VarField(String),
    VarTimestamp(String),
    VarCustom(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZervVars {
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub tag_timestamp: Option<u64>,
    pub tag_branch: Option<String>,
    pub current_branch: Option<String>,
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub tag_commit_hash: Option<String>,
    pub current_commit_hash: Option<String>,
    pub epoch: Option<u64>,
    pub pre_release: Option<PreReleaseVar>,
    pub post: Option<u64>,
    pub dev: Option<u64>,
    pub custom: HashMap<String, VarValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreReleaseVar {
    pub label: PreReleaseLabel,
    pub number: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarValue {
    String(String),
    Integer(u64),
    Boolean(bool),
}

impl Zerv {
    pub fn new(format: ZervFormat, vars: ZervVars) -> Self {
        Self { format, vars }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod construction {
        use super::*;

        #[test]
        fn test_zerv_new() {
            let format = ZervFormat {
                core: vec![Component::VarField("major".to_string())],
                extra_core: vec![],
                build: vec![],
            };
            let vars = ZervVars::default();
            let zerv = Zerv::new(format.clone(), vars.clone());

            assert_eq!(zerv.format, format);
            assert_eq!(zerv.vars, vars);
        }

        #[test]
        fn test_zerv_vars_default() {
            let vars = ZervVars::default();

            assert_eq!(vars.major, None);
            assert_eq!(vars.minor, None);
            assert_eq!(vars.patch, None);
            assert!(vars.custom.is_empty());
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

        #[test]
        fn test_component_var_custom() {
            let comp = Component::VarCustom("custom_var".to_string());
            match comp {
                Component::VarCustom(key) => assert_eq!(key, "custom_var"),
                _ => panic!("Expected VarCustom component"),
            }
        }
    }

    mod var_values {
        use super::*;

        #[test]
        fn test_var_value_string() {
            let val = VarValue::String("test".to_string());
            match val {
                VarValue::String(s) => assert_eq!(s, "test"),
                _ => panic!("Expected String value"),
            }
        }

        #[test]
        fn test_var_value_integer() {
            let val = VarValue::Integer(123);
            match val {
                VarValue::Integer(n) => assert_eq!(n, 123),
                _ => panic!("Expected Integer value"),
            }
        }

        #[test]
        fn test_var_value_boolean() {
            let val = VarValue::Boolean(true);
            match val {
                VarValue::Boolean(b) => assert!(b),
                _ => panic!("Expected Boolean value"),
            }
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
                current_branch: Some("main".to_string()),
                dirty: Some(true),
                ..Default::default()
            };

            assert_eq!(vars.major, Some(1));
            assert_eq!(vars.minor, Some(2));
            assert_eq!(vars.patch, Some(3));
            assert_eq!(vars.current_branch, Some("main".to_string()));
            assert_eq!(vars.dirty, Some(true));
        }

        #[test]
        fn test_custom_variables() {
            let mut vars = ZervVars::default();
            vars.custom
                .insert("build_id".to_string(), VarValue::Integer(456));
            vars.custom
                .insert("env".to_string(), VarValue::String("prod".to_string()));
            vars.custom
                .insert("debug".to_string(), VarValue::Boolean(false));

            assert_eq!(vars.custom.get("build_id"), Some(&VarValue::Integer(456)));
            assert_eq!(
                vars.custom.get("env"),
                Some(&VarValue::String("prod".to_string()))
            );
            assert_eq!(vars.custom.get("debug"), Some(&VarValue::Boolean(false)));
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_format() {
            let format = ZervFormat {
                core: vec![],
                extra_core: vec![],
                build: vec![],
            };
            let vars = ZervVars::default();
            let zerv = Zerv::new(format, vars);

            assert!(zerv.format.core.is_empty());
            assert!(zerv.format.extra_core.is_empty());
            assert!(zerv.format.build.is_empty());
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
                current_branch: Some("".to_string()),
                tag_commit_hash: Some("".to_string()),
                ..Default::default()
            };

            assert_eq!(vars.current_branch, Some("".to_string()));
            assert_eq!(vars.tag_commit_hash, Some("".to_string()));
        }
    }

    mod complex_structures {
        use super::*;

        #[test]
        fn test_semver_like_structure() {
            let format = ZervFormat {
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

            let zerv = Zerv::new(format, vars);
            assert_eq!(zerv.vars.major, Some(1));
            assert_eq!(
                zerv.vars.pre_release.as_ref().unwrap().label,
                PreReleaseLabel::Alpha
            );
        }

        #[test]
        fn test_calver_like_structure() {
            let format = ZervFormat {
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
                tag_timestamp: Some(1710547200),
                ..Default::default()
            };

            let zerv = Zerv::new(format, vars);
            assert_eq!(zerv.vars.patch, Some(1));
            assert_eq!(zerv.vars.tag_timestamp, Some(1710547200));
        }
    }
}
