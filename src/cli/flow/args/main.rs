use std::str::FromStr;

use clap::Parser;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
    Validation as CommonValidation,
};
use crate::cli::flow::branch_rules::BranchRules;
use crate::cli::utils::template::Template;
use crate::error::ZervError;
use crate::schema::ZervSchemaPreset;
use crate::utils::constants::pre_release_labels::ALPHA;
use crate::utils::constants::{
    post_modes,
    pre_release_labels,
};

/// Generate version with intelligent pre-release management based on Git branch patterns
#[derive(Parser)]
#[command(
    about = "Generate version with intelligent pre-release management based on Git branch patterns"
)]
#[command(
    long_about = "Generate version strings with automatic pre-release detection based on Git branch patterns.
This command acts as an intelligent wrapper around 'zerv version' that automatically determines
pre-release information from the current Git branch.

INPUT/OUTPUT OPTIONS:
  -s, --source <TYPE>       Input source: git, stdin
  -f, --input-format <FMT>  Input format: auto, semver, pep440
  -o, --output-format <FMT> Output format: semver, pep440, zerv
  -t, --output-template <TPL> Custom output template (Handlebars)
  -p, --output-prefix <PFX> Add prefix to version output

PRE-RELEASE OPTIONS:
  --pre-release-label <LBL> Pre-release label: alpha (default), beta, rc
  --pre-release-num <NUM>   Pre-release number: integer (default: {{hash_int bumped_branch HASH_BRANCH_LEN}})
  --hash-branch-len <LEN>   Hash length for bumped branch hash (1-10, default: 5)

POST MODE OPTIONS:
  --post-mode <MODE>        Post calculation mode: commit (default), tag

SCHEMA OPTIONS:
  --schema <SCHEMA>         Schema variant for output components [default: standard] [possible values: standard, standard-no-context, standard-context, standard-base, standard-base-prerelease, standard-base-prerelease-post, standard-base-prerelease-post-dev]

EXAMPLES:
  # Basic flow version with automatic pre-release detection (smart schema)
  zerv flow

  # Different output formats
  zerv flow --output-format pep440
  zerv flow --output-format zerv
  zerv flow --output-prefix v

  # Pre-release control
  zerv flow --pre-release-label beta
  zerv flow --pre-release-label rc --pre-release-num 5

  # Post mode control
  zerv flow --post-mode commit  # bump post by distance (default)
  zerv flow --post-mode tag     # bump post by 1

  # Schema control (replaces --dev-ts, --no-dev-ts, --no-pre-release flags)
  zerv flow --schema standard              # smart context (default)
  zerv flow --schema standard-no-context   # never include context
  zerv flow --schema standard-context      # always include context
  zerv flow --schema standard-base         # base version only
  zerv flow --schema standard-base-prerelease-post  # prerelease + post only

  # Use in different directory
  zerv flow -C /path/to/repo"
)]
#[derive(Debug)]
pub struct FlowArgs {
    #[command(flatten)]
    pub input: InputConfig,

    #[command(flatten)]
    pub output: OutputConfig,

    /// Pre-release label for flow versions (alpha, beta, rc)
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(pre_release_labels::VALID_LABELS),
          help = "Pre-release label for flow versions (alpha, beta, rc)")]
    pub pre_release_label: Option<String>,

    #[arg(
        long,
        value_parser = clap::value_parser!(u32),
        help = "Pre-release number for flow versions (integer, default: {{ hash_int(value=bumped_branch, length=HASH_BRANCH_LEN) }})"
    )]
    pub pre_release_num: Option<u32>,

    #[arg(
        long = "hash-branch-len",
        value_parser = clap::value_parser!(u32),
        default_value = "5",
        help = "Hash length for bumped branch hash (1-10, default: 5)"
    )]
    pub hash_branch_len: u32,

    /// Post calculation mode (commit, tag)
    #[arg(long = "post-mode", value_parser = clap::builder::PossibleValuesParser::new(post_modes::VALID_MODES),
          default_value = post_modes::COMMIT, help = "Post calculation mode (commit, tag)")]
    pub post_mode: String,

    /// Branch rules in RON format (default: GitFlow rules)
    #[arg(
        long = "branch-rules",
        help = "Branch rules in RON format (default: GitFlow rules)",
        value_parser = clap::value_parser!(BranchRules),
        default_value_t = BranchRules::default_rules(),
    )]
    pub branch_rules: BranchRules,

    /// Schema variant for output components [default: standard]
    #[arg(
        long,
        help = "Schema variant for output components [default: standard] [possible values: standard, standard-no-context, standard-context, standard-base, standard-base-prerelease, standard-base-prerelease-post, standard-base-prerelease-post-dev]"
    )]
    pub schema: Option<String>,
}

impl Default for FlowArgs {
    fn default() -> Self {
        Self {
            input: InputConfig::default(),
            output: OutputConfig::default(),
            pre_release_label: None,
            pre_release_num: None,
            hash_branch_len: 5,
            post_mode: post_modes::COMMIT.to_string(),
            branch_rules: BranchRules::default_rules(),
            schema: None,
        }
    }
}

impl FlowArgs {
    pub fn build_patch_bump_template(&self, content: &str) -> String {
        let if_part = "{% if not pre_release and (dirty or distance) %}";
        let else_part = "{% else %}None{% endif %}";
        if_part.to_string() + content + else_part
    }
    pub fn build_pre_release_bump_template(&self, content: &str) -> String {
        let if_part = "{% if dirty or distance %}";
        let else_part = "{% else %}None{% endif %}";
        if_part.to_string() + content + else_part
    }

    pub fn validate(&mut self) -> Result<(), ZervError> {
        // Use shared validation for input/output
        CommonValidation::validate_io(&self.input, &self.output)?;

        self.validate_pre_release_label()?;
        self.validate_pre_release_num()?;
        self.validate_hash_branch_len()?;
        self.validate_post_mode()?;
        self.validate_schema()?;

        Ok(())
    }

    fn validate_pre_release_label(&mut self) -> Result<(), ZervError> {
        if self.pre_release_label.is_none() {
            self.pre_release_label = Some(ALPHA.to_string());
        }
        Ok(())
    }

    fn validate_pre_release_num(&self) -> Result<(), ZervError> {
        // No longer need to check for no_pre_release conflicts
        Ok(())
    }

    fn validate_hash_branch_len(&self) -> Result<(), ZervError> {
        if self.hash_branch_len == 0 || self.hash_branch_len > 10 {
            return Err(ZervError::InvalidArgument(format!(
                "hash-branch-len must be between 1 and 10, got {}",
                self.hash_branch_len
            )));
        }
        Ok(())
    }

    fn validate_post_mode(&self) -> Result<(), ZervError> {
        if !post_modes::VALID_MODES.contains(&self.post_mode.as_str()) {
            return Err(ZervError::InvalidArgument(format!(
                "post-mode must be one of: {}, got {}",
                post_modes::VALID_MODES.join(", "),
                self.post_mode
            )));
        }
        Ok(())
    }

    fn validate_schema(&self) -> Result<(), ZervError> {
        if let Some(schema_name) = &self.schema {
            // First, validate it's a known schema
            let _parsed = ZervSchemaPreset::from_str(schema_name).map_err(|_| {
                ZervError::InvalidArgument(format!("Unknown schema variant: '{}'", schema_name))
            })?;

            // Restrict to standard schemas only (check prefix)
            if schema_name.starts_with("standard") {
                Ok(())
            } else {
                Err(ZervError::InvalidArgument(format!(
                    "zerv flow only supports standard schema variants, got: '{}'",
                    schema_name
                )))
            }
        } else {
            Ok(())
        }
    }

    pub fn bump_pre_release_label(&self) -> Option<Template<String>> {
        self.pre_release_label.clone().map(|label| {
            let template = self.build_pre_release_bump_template(&label);
            Template::new(template)
        })
    }

    pub fn bump_pre_release_num(&self) -> Option<Option<Template<u32>>> {
        if self.pre_release_label.is_none() {
            None
        } else {
            let hash_len = self.hash_branch_len.to_string();

            let pre_release_num_content = if let Some(num) = self.pre_release_num {
                num.to_string()
            } else {
                format!(
                    "{{{{ hash_int(value=bumped_branch, length={}) }}}}",
                    hash_len
                )
            };

            let template = self.build_pre_release_bump_template(&pre_release_num_content);

            Some(Some(Template::new(template)))
        }
    }

    pub fn bump_patch(&self) -> Option<Option<Template<u32>>> {
        let template = self.build_patch_bump_template("1");
        Some(Some(Template::new(template)))
    }

    pub fn bump_post(&self) -> Option<Option<Template<u32>>> {
        let content = match self.post_mode.as_str() {
            post_modes::COMMIT => "{{ distance }}", // bump post by distance
            post_modes::TAG => "1",                 // bump post by 1
            _ => unreachable!("Invalid post_mode should have been caught by validation"),
        };
        let template = self.build_pre_release_bump_template(content);
        Some(Some(Template::new(template)))
    }

    pub fn bump_dev(&self) -> Option<Option<Template<u32>>> {
        let if_part = "{% if dirty %}";
        let content = "{{ bumped_timestamp }}";
        let else_part = "{% else %}None{% endif %}";
        let template = format!("{}{}{}", if_part, content, else_part);
        Some(Some(Template::new(template)))
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::cli::common::args::{
        InputConfig,
        OutputConfig,
    };

    mod defaults {
        use super::*;

        #[test]
        fn test_flow_args_default() {
            let args = FlowArgs::default();
            assert_eq!(args.input.source, "git");
            assert_eq!(args.output.output_format, "semver");
            assert_eq!(args.hash_branch_len, 5);
            assert!(args.pre_release_label.is_none());
            assert!(args.pre_release_num.is_none());
            assert_eq!(args.post_mode, post_modes::COMMIT);
            assert!(args.schema.is_none()); // Default is None (will use standard schema)
        }

        #[test]
        fn test_flow_args_validation_success() {
            let mut args = FlowArgs::default();
            assert!(args.validate().is_ok());
            assert!(args.schema.is_none()); // Should remain None after validation
        }
    }

    mod validation {
        use super::*;

        #[rstest]
        #[case("alpha")]
        #[case("beta")]
        #[case("rc")]
        fn test_valid_pre_release_labels(#[case] label: &str) {
            let mut args = FlowArgs {
                pre_release_label: Some(label.to_string()),
                ..FlowArgs::default()
            };
            assert!(args.validate().is_ok());
        }

        #[rstest]
        #[case(1)]
        #[case(5)]
        #[case(10)]
        fn test_valid_hash_branch_lengths(#[case] length: u32) {
            let mut args = FlowArgs {
                hash_branch_len: length,
                ..FlowArgs::default()
            };
            assert!(args.validate().is_ok());
        }

        #[rstest]
        #[case(post_modes::COMMIT)]
        #[case(post_modes::TAG)]
        fn test_valid_post_modes(#[case] mode: &str) {
            let mut args = FlowArgs {
                post_mode: mode.to_string(),
                ..FlowArgs::default()
            };
            assert!(args.validate().is_ok());
        }

        #[rstest]
        #[case(0)]
        #[case(11)]
        #[case(20)]
        fn test_invalid_hash_branch_lengths(#[case] length: u32) {
            let mut args = FlowArgs {
                hash_branch_len: length,
                ..FlowArgs::default()
            };
            let result = args.validate();
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("hash-branch-len must be between 1 and 10")
            );
        }

        #[rstest]
        #[case("invalid")]
        #[case("commit-invalid")]
        #[case("tag-invalid")]
        #[case("")]
        fn test_invalid_post_modes(#[case] mode: &str) {
            let mut args = FlowArgs {
                post_mode: mode.to_string(),
                ..FlowArgs::default()
            };
            let result = args.validate();
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("post-mode must be one of:")
            );
        }

        #[rstest]
        #[case(Some("beta".to_string()), Some(5))]
        #[case(None, None)]
        #[case(Some("rc".to_string()), None)]
        #[case(None, Some(10))]
        fn test_valid_combinations(#[case] label: Option<String>, #[case] num: Option<u32>) {
            let mut args = FlowArgs {
                pre_release_label: label,
                pre_release_num: num,
                ..FlowArgs::default()
            };
            assert!(args.validate().is_ok());
        }
    }

    mod input_output {
        use super::*;

        #[test]
        fn test_flow_args_with_custom_input_output() {
            let mut args = FlowArgs {
                input: InputConfig {
                    source: "git".to_string(),
                    input_format: "auto".to_string(),
                    directory: Some("/test/path".to_string()),
                },
                output: OutputConfig {
                    output_format: "zerv".to_string(),
                    output_prefix: Some("v".to_string()),
                    output_template: None,
                },
                ..FlowArgs::default()
            };
            assert_eq!(args.input.source, "git");
            assert_eq!(args.output.output_format, "zerv");
            assert_eq!(args.output.output_prefix, Some("v".to_string()));
            assert!(args.validate().is_ok());
        }
    }

    mod bump_pre_release_label {
        use super::*;

        #[test]
        fn test_default_returns_alpha() {
            let mut args = FlowArgs::default();
            args.validate().unwrap(); // This sets the default pre_release_label
            let expected = args.build_pre_release_bump_template("alpha");
            assert_eq!(args.bump_pre_release_label(), Some(Template::new(expected)));
        }

        #[rstest]
        #[case("beta")]
        #[case("rc")]
        fn test_custom_label_returned(#[case] label: &str) {
            let args = FlowArgs {
                pre_release_label: Some(label.to_string()),
                ..FlowArgs::default()
            };
            let expected = args.build_pre_release_bump_template(label);
            assert_eq!(args.bump_pre_release_label(), Some(Template::new(expected)));
        }
    }

    mod bump_pre_release_num {
        use super::*;

        #[test]
        fn test_default_returns_template() {
            let args = FlowArgs {
                pre_release_label: Some("alpha".to_string()), // Need a pre-release label for bump_pre_release_num to return template
                ..FlowArgs::default()
            };
            let result = args.bump_pre_release_num();
            assert!(result.is_some());
            assert!(result.unwrap().is_some()); // Should be Some(Template)
        }

        #[rstest]
        #[case(5)]
        #[case(123)]
        #[case(999)]
        fn test_custom_num_returns_value(#[case] num: u32) {
            let args = FlowArgs {
                pre_release_label: Some("alpha".to_string()), // Need a pre-release label for bump_pre_release_num to return template
                pre_release_num: Some(num),
                ..FlowArgs::default()
            };
            let result = args.bump_pre_release_num();
            assert!(result.is_some());
            let template = result.unwrap().unwrap();

            // Generate expected template using the helper function
            let expected = args.build_pre_release_bump_template(&num.to_string());
            assert_eq!(template.as_str(), expected);
        }

        #[rstest]
        #[case(3)]
        #[case(7)]
        #[case(5)]
        fn test_template_uses_hash_branch_len(#[case] length: u32) {
            let args = FlowArgs {
                pre_release_label: Some("alpha".to_string()), // Need a pre-release label for bump_pre_release_num to return template
                hash_branch_len: length,
                ..FlowArgs::default()
            };
            let result = args.bump_pre_release_num();
            assert!(result.is_some());
            let template = result.unwrap().unwrap();

            // Generate expected template from input
            let content = format!("{{{{ hash_int(value=bumped_branch, length={}) }}}}", length);
            let expected = args.build_pre_release_bump_template(&content);
            assert_eq!(template.as_str(), expected);
        }
    }

    mod error_cases {
        use super::*;

        #[rstest]
        #[case("invalid")]
        #[case("dev")]
        #[case("alpha1")]
        #[case("alpha-beta")]
        #[case("ALPHA")]
        #[case("")]
        fn test_invalid_pre_release_label_choices(#[case] invalid_label: &str) {
            // Note: clap's PossibleValuesParser handles validation at parsing time
            // This test demonstrates what happens when invalid values are somehow set
            let mut args = FlowArgs {
                pre_release_label: Some(invalid_label.to_string()),
                ..FlowArgs::default()
            };

            // Note: Our current validation only checks structural constraints, not value constraints
            // The clap argument parser would catch invalid values before they reach validate()
            // This test shows the validation passes but the values would be rejected by clap
            assert!(args.validate().is_ok());
        }
    }

    mod bump_post {
        use super::*;

        #[rstest]
        #[case(post_modes::COMMIT, "{{ distance }}")]
        #[case(post_modes::TAG, "1")]
        fn test_bump_post_templates(#[case] mode: &str, #[case] expected_content: &str) {
            let args = FlowArgs {
                post_mode: mode.to_string(),
                pre_release_label: Some("alpha".to_string()), // Need a pre-release label
                ..FlowArgs::default()
            };

            let result = args.bump_post();
            assert!(result.is_some());
            let template_result = result.unwrap();
            assert!(template_result.is_some());
            let template = template_result.unwrap();

            let expected = args.build_pre_release_bump_template(expected_content);
            assert_eq!(template.as_str(), expected);
        }

        #[test]
        fn test_bump_post_commit_mode_default() {
            let args = FlowArgs::default();
            let result = args.bump_post();
            assert!(result.is_some());

            let template_result = result.unwrap();
            assert!(template_result.is_some());
            let template = template_result.unwrap();
            let expected = args.build_pre_release_bump_template("{{ distance }}");
            assert_eq!(template.as_str(), expected);
        }

        #[test]
        #[should_panic(expected = "Invalid post_mode should have been caught by validation")]
        fn test_bump_post_invalid_mode_panics() {
            let args = FlowArgs {
                post_mode: "invalid".to_string(),
                ..FlowArgs::default()
            };

            // This should panic because invalid post_mode should be caught by validation
            args.bump_post();
        }
    }

    mod bump_dev {
        use super::*;

        #[test]
        fn test_bump_dev_always_returns_template() {
            let args = FlowArgs::default();
            let result = args.bump_dev();
            assert!(result.is_some());
            let template_result = result.unwrap();
            assert!(template_result.is_some());
            let template = template_result.unwrap();

            assert_eq!(
                template.as_str(),
                "{% if dirty %}{{ bumped_timestamp }}{% else %}None{% endif %}"
            );
        }
    }

    mod schema_validation {
        use super::*;

        #[rstest]
        #[case("standard")]
        #[case("standard-no-context")]
        #[case("standard-context")]
        #[case("standard-base")]
        #[case("standard-base-prerelease")]
        #[case("standard-base-prerelease-post")]
        #[case("standard-base-prerelease-post-dev")]
        #[case("standard-base-context")]
        #[case("standard-base-prerelease-context")]
        #[case("standard-base-prerelease-post-context")]
        #[case("standard-base-prerelease-post-dev-context")]
        fn test_valid_standard_schemas(#[case] schema: &str) {
            let mut args = FlowArgs {
                schema: Some(schema.to_string()),
                ..FlowArgs::default()
            };
            assert!(args.validate().is_ok());
        }

        #[rstest]
        #[case("calver")]
        #[case("calver-base")]
        #[case("calver-context")]
        #[case("calver-no-context")]
        #[case("calver-base-prerelease")]
        #[case("invalid-schema")]
        #[case("unknown")]
        #[case("")]
        fn test_invalid_schemas(#[case] schema: &str) {
            let mut args = FlowArgs {
                schema: Some(schema.to_string()),
                ..FlowArgs::default()
            };
            let result = args.validate();
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();

            // Both error types are valid - unknown schemas or non-standard schemas
            assert!(
                error_msg.contains("Unknown schema variant")
                    || error_msg.contains("zerv flow only supports standard schema variants")
            );
        }

        #[test]
        fn test_no_schema_defaults_to_none() {
            let args = FlowArgs::default();
            assert!(args.schema.is_none());

            let mut args = args;
            assert!(args.validate().is_ok());
            assert!(args.schema.is_none()); // Should remain None
        }

        #[test]
        fn test_schema_validation_with_pre_release_overrides() {
            let mut args = FlowArgs {
                schema: Some("standard-base".to_string()),
                pre_release_label: Some("beta".to_string()),
                pre_release_num: Some(42),
                ..FlowArgs::default()
            };

            // Should validate successfully - schema and manual overrides can coexist
            assert!(args.validate().is_ok());
        }
    }

    mod branch_rules {
        use super::*;

        #[test]
        fn test_flow_args_default_has_gitflow_rules() {
            let args = FlowArgs::default();
            // Should have default GitFlow rules
            assert!(args.branch_rules.find_rule("develop").is_some());
            assert!(args.branch_rules.find_rule("release/1").is_some());
        }

        #[test]
        fn test_flow_args_with_custom_branch_rules() {
            let custom_ron = r#"[
                (pattern: "main", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
                (pattern: "hotfix/*", pre_release_label: rc, post_mode: tag)
            ]"#;

            let custom_rules: BranchRules = custom_ron.parse().unwrap();
            let args = FlowArgs {
                branch_rules: custom_rules,
                ..FlowArgs::default()
            };

            // Should have custom rules, not default GitFlow rules
            assert!(args.branch_rules.find_rule("main").is_some());
            assert!(args.branch_rules.find_rule("hotfix/123").is_some());
            assert!(args.branch_rules.find_rule("develop").is_none()); // Not in custom rules
            assert!(args.branch_rules.find_rule("release/1").is_none()); // Not in custom rules
        }

        #[test]
        fn test_flow_args_validation_with_custom_branch_rules() {
            let custom_ron = r#"[
                (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
                (pattern: "release/*", pre_release_label: rc, post_mode: tag)
            ]"#;

            let mut args = FlowArgs {
                branch_rules: custom_ron.parse().unwrap(),
                ..FlowArgs::default()
            };

            // Validation should succeed even with custom branch rules
            assert!(args.validate().is_ok());
        }
    }

    mod integration {
        use super::*;

        #[rstest]
        #[case("alpha", Some(5), 3, post_modes::COMMIT)]
        #[case("beta", None, 7, post_modes::TAG)]
        #[case("rc", Some(10), 1, post_modes::COMMIT)]
        fn test_complete_configuration(
            #[case] label: &str,
            #[case] num: Option<u32>,
            #[case] hash_len: u32,
            #[case] post_mode: &str,
        ) {
            let mut args = FlowArgs {
                pre_release_label: Some(label.to_string()),
                pre_release_num: num,
                hash_branch_len: hash_len,
                post_mode: post_mode.to_string(),
                ..FlowArgs::default()
            };

            // Test validation
            assert!(args.validate().is_ok());

            // Test bump_pre_release_label
            let expected = args.build_pre_release_bump_template(label);
            assert_eq!(args.bump_pre_release_label(), Some(Template::new(expected)));

            // Test bump_pre_release_num
            let result = args.bump_pre_release_num();
            assert!(result.is_some());

            if let Some(num_value) = num {
                let template = result.unwrap().unwrap();
                let expected = args.build_pre_release_bump_template(&num_value.to_string());
                assert_eq!(template.as_str(), expected);
            } else {
                let template = result.unwrap().unwrap();
                let content = format!(
                    "{{{{ hash_int(value=bumped_branch, length={}) }}}}",
                    hash_len
                );
                let expected = args.build_pre_release_bump_template(&content);
                assert_eq!(template.as_str(), expected);
            }

            // Test bump_post with the specified post_mode
            let post_result = args.bump_post();
            assert!(post_result.is_some());
            let post_template_result = post_result.unwrap();
            assert!(post_template_result.is_some());
            let post_template = post_template_result.unwrap();

            let expected_post_content = match post_mode {
                post_modes::COMMIT => "{{ distance }}",
                post_modes::TAG => "1",
                _ => "{{ distance }}",
            };
            let expected_post = args.build_pre_release_bump_template(expected_post_content);
            assert_eq!(post_template.as_str(), expected_post);
        }

        #[test]
        fn test_integration_with_branch_rules_and_manual_overrides() {
            let custom_ron = r#"[
                (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
                (pattern: "release/*", pre_release_label: rc, post_mode: tag)
            ]"#;

            let mut args = FlowArgs {
                branch_rules: custom_ron.parse().unwrap(),
                pre_release_label: Some("alpha".to_string()), // Manual override
                pre_release_num: Some(42),                    // Manual override
                post_mode: "tag".to_string(),                 // Manual override
                ..FlowArgs::default()
            };

            // Should validate successfully - branch rules and manual overrides can coexist
            assert!(args.validate().is_ok());

            // Manual overrides should be preserved
            assert_eq!(args.pre_release_label, Some("alpha".to_string()));
            assert_eq!(args.pre_release_num, Some(42));
            assert_eq!(args.post_mode, "tag");

            // Branch rules should still be available for validation logic
            assert!(args.branch_rules.find_rule("develop").is_some());
            assert!(args.branch_rules.find_rule("release/1").is_some());
        }
    }
}
