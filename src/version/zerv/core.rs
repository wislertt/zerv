use crate::constants::pre_release_labels;
use crate::error::ZervError;
use crate::version::zerv::schema::ZervSchema;
use crate::version::zerv::vars::ZervVars;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreReleaseLabel {
    Alpha,
    Beta,
    Rc,
}

impl PreReleaseLabel {
    /// Get string representation of the label
    pub fn label_str(&self) -> &'static str {
        match self {
            PreReleaseLabel::Alpha => pre_release_labels::ALPHA,
            PreReleaseLabel::Beta => pre_release_labels::BETA,
            PreReleaseLabel::Rc => pre_release_labels::RC,
        }
    }

    /// Get all valid label strings
    pub fn valid_labels() -> &'static [&'static str] {
        pre_release_labels::VALID_LABELS
    }

    /// Flexible parsing with alternative forms
    /// This replaces the existing normalize_pre_release_label function
    pub fn try_from_str(label: &str) -> Option<Self> {
        match label.to_lowercase().as_str() {
            pre_release_labels::ALPHA | "a" => Some(PreReleaseLabel::Alpha),
            pre_release_labels::BETA | "b" => Some(PreReleaseLabel::Beta),
            pre_release_labels::RC | "c" | "preview" | "pre" => Some(PreReleaseLabel::Rc),
            _ => None,
        }
    }

    /// Flexible parsing with alpha fallback (for PEP440 parser compatibility)
    /// This replaces the existing normalize_pre_label function
    pub fn from_str_or_alpha(label: &str) -> Self {
        Self::try_from_str(label).unwrap_or(PreReleaseLabel::Alpha)
    }
}

impl FromStr for PreReleaseLabel {
    type Err = ZervError;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        match label {
            pre_release_labels::ALPHA => Ok(PreReleaseLabel::Alpha),
            pre_release_labels::BETA => Ok(PreReleaseLabel::Beta),
            pre_release_labels::RC => Ok(PreReleaseLabel::Rc),
            _ => Err(ZervError::InvalidPreReleaseLabel(format!(
                "Invalid pre-release label '{}'. Valid labels: {:?}",
                label,
                pre_release_labels::VALID_LABELS
            ))),
        }
    }
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
        fn test_pre_release_label_label_str() {
            assert_eq!(PreReleaseLabel::Alpha.label_str(), "alpha");
            assert_eq!(PreReleaseLabel::Beta.label_str(), "beta");
            assert_eq!(PreReleaseLabel::Rc.label_str(), "rc");
        }

        #[test]
        fn test_pre_release_label_valid_labels() {
            let valid_labels = PreReleaseLabel::valid_labels();
            assert_eq!(valid_labels, &["alpha", "beta", "rc"]);
        }

        #[test]
        fn test_pre_release_label_from_str() {
            assert_eq!(
                "alpha".parse::<PreReleaseLabel>().unwrap(),
                PreReleaseLabel::Alpha
            );
            assert_eq!(
                "beta".parse::<PreReleaseLabel>().unwrap(),
                PreReleaseLabel::Beta
            );
            assert_eq!(
                "rc".parse::<PreReleaseLabel>().unwrap(),
                PreReleaseLabel::Rc
            );

            assert!("invalid".parse::<PreReleaseLabel>().is_err());
            assert!("gamma".parse::<PreReleaseLabel>().is_err());
        }

        #[test]
        fn test_pre_release_label_try_from_str() {
            // Valid exact matches
            assert_eq!(
                PreReleaseLabel::try_from_str("alpha"),
                Some(PreReleaseLabel::Alpha)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("beta"),
                Some(PreReleaseLabel::Beta)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("rc"),
                Some(PreReleaseLabel::Rc)
            );

            // Valid alternative forms
            assert_eq!(
                PreReleaseLabel::try_from_str("a"),
                Some(PreReleaseLabel::Alpha)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("b"),
                Some(PreReleaseLabel::Beta)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("c"),
                Some(PreReleaseLabel::Rc)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("preview"),
                Some(PreReleaseLabel::Rc)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("pre"),
                Some(PreReleaseLabel::Rc)
            );

            // Case insensitive
            assert_eq!(
                PreReleaseLabel::try_from_str("ALPHA"),
                Some(PreReleaseLabel::Alpha)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("Beta"),
                Some(PreReleaseLabel::Beta)
            );
            assert_eq!(
                PreReleaseLabel::try_from_str("RC"),
                Some(PreReleaseLabel::Rc)
            );

            // Invalid
            assert_eq!(PreReleaseLabel::try_from_str("invalid"), None);
            assert_eq!(PreReleaseLabel::try_from_str("gamma"), None);
        }

        #[test]
        fn test_pre_release_label_from_str_or_alpha() {
            // Valid labels
            assert_eq!(
                PreReleaseLabel::from_str_or_alpha("beta"),
                PreReleaseLabel::Beta
            );
            assert_eq!(
                PreReleaseLabel::from_str_or_alpha("rc"),
                PreReleaseLabel::Rc
            );

            // Invalid labels default to Alpha
            assert_eq!(
                PreReleaseLabel::from_str_or_alpha("invalid"),
                PreReleaseLabel::Alpha
            );
            assert_eq!(
                PreReleaseLabel::from_str_or_alpha("gamma"),
                PreReleaseLabel::Alpha
            );
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
