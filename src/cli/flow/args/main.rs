use clap::Parser;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
    Validation as CommonValidation,
};
use crate::cli::utils::template::Template;
use crate::error::ZervError;
use crate::utils::constants::pre_release_labels;
use crate::utils::constants::pre_release_labels::ALPHA;

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
  --no-pre-release          Disable pre-release entirely

EXAMPLES:
  # Basic flow version with automatic pre-release detection
  zerv flow

  # Different output formats
  zerv flow --output-format pep440
  zerv flow --output-format zerv
  zerv flow --output-prefix v

  # Pre-release control
  zerv flow --pre-release-label beta
  zerv flow --pre-release-label rc --pre-release-num 5
  zerv flow --no-pre-release

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
        help = "Pre-release number for flow versions (integer, default: {{hash_int bumped_branch HASH_BRANCH_LEN}})"
    )]
    pub pre_release_num: Option<u32>,

    #[arg(
        long = "hash-branch-len",
        value_parser = clap::value_parser!(u32),
        default_value = "5",
        help = "Hash length for bumped branch hash (1-10, default: 5)"
    )]
    pub hash_branch_len: u32,

    /// Disable pre-release entirely (no pre-release component)
    #[arg(long, action = clap::ArgAction::SetTrue,
          help = "Disable pre-release entirely (no pre-release component)")]
    pub no_pre_release: bool,
}

impl Default for FlowArgs {
    fn default() -> Self {
        Self {
            input: InputConfig::default(),
            output: OutputConfig::default(),
            pre_release_label: None,
            pre_release_num: None,
            hash_branch_len: 5,
            no_pre_release: false,
        }
    }
}

impl FlowArgs {
    pub fn validate(&mut self) -> Result<(), ZervError> {
        // Use shared validation for input/output
        CommonValidation::validate_io(&self.input, &self.output)?;

        self.validate_pre_release_label()?;
        self.validate_pre_release_num()?;
        self.validate_hash_branch_len()?;

        Ok(())
    }

    fn validate_pre_release_label(&mut self) -> Result<(), ZervError> {
        if self.no_pre_release && self.pre_release_label.is_some() {
            return Err(ZervError::InvalidArgument(
                "Cannot use --pre-release-label with --no-pre-release".to_string(),
            ));
        } else if !self.no_pre_release && self.pre_release_label.is_none() {
            self.pre_release_label = Some(ALPHA.to_string());
        }
        Ok(())
    }

    fn validate_pre_release_num(&self) -> Result<(), ZervError> {
        if self.no_pre_release && self.pre_release_num.is_some() {
            return Err(ZervError::InvalidArgument(
                "Cannot use --pre-release-num with --no-pre-release".to_string(),
            ));
        }
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

    pub fn bump_pre_release_label(&self) -> Option<Template<String>> {
        self.pre_release_label.clone().map(Template::new)
    }

    pub fn bump_pre_release_num(&self) -> Option<Option<Template<u32>>> {
        if self.no_pre_release || self.pre_release_label.is_none() {
            None
        } else {
            // Some(self.pre_release_num.map(Template::Value).or_else(|| {
            //     Some(Template::Template(format!(
            //         "{{{{hash_int bumped_branch {}}}}}",
            //         self.hash_branch_len
            //     )))
            // }))
            // Some(Some(Template::Template(format!(
            //     "{{{{hash_int bumped_branch {}}}}}",
            //     self.hash_branch_len
            // ))))
            Some(Some(Template::new(
                "{{#if (and pre_release (or dirty distance))}}
                    {{hash_int bumped_branch 5}}
                {{else}}
                    0
                {{/if}}"
                    .to_string(),
            )))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use rstest::*;

//     use super::*;
//     use crate::cli::common::args::{
//         InputConfig,
//         OutputConfig,
//     };

//     mod defaults {
//         use super::*;

//         #[test]
//         fn test_flow_args_default() {
//             let args = FlowArgs::default();
//             assert_eq!(args.input.source, "git");
//             assert_eq!(args.output.output_format, "semver");
//             assert_eq!(args.hash_branch_len, 5);
//             assert!(args.pre_release_label.is_none());
//             assert!(args.pre_release_num.is_none());
//             assert!(!args.no_pre_release);
//         }

//         #[test]
//         fn test_flow_args_validation_success() {
//             let mut args = FlowArgs::default();
//             assert!(args.validate().is_ok());
//         }
//     }

//     mod validation {
//         use super::*;

//         #[rstest]
//         #[case("alpha")]
//         #[case("beta")]
//         #[case("rc")]
//         fn test_valid_pre_release_labels(#[case] label: &str) {
//             let mut args = FlowArgs {
//                 pre_release_label: Some(label.to_string()),
//                 ..FlowArgs::default()
//             };
//             assert!(args.validate().is_ok());
//         }

//         #[rstest]
//         #[case(1)]
//         #[case(5)]
//         #[case(10)]
//         fn test_valid_hash_branch_lengths(#[case] length: u32) {
//             let mut args = FlowArgs {
//                 hash_branch_len: length,
//                 ..FlowArgs::default()
//             };
//             assert!(args.validate().is_ok());
//         }

//         #[rstest]
//         #[case(0)]
//         #[case(11)]
//         #[case(20)]
//         fn test_invalid_hash_branch_lengths(#[case] length: u32) {
//             let mut args = FlowArgs {
//                 hash_branch_len: length,
//                 ..FlowArgs::default()
//             };
//             let result = args.validate();
//             assert!(result.is_err());
//             assert!(
//                 result
//                     .unwrap_err()
//                     .to_string()
//                     .contains("hash-branch-len must be between 1 and 10")
//             );
//         }

//         #[rstest]
//         #[case(Some("beta".to_string()), None, true, "Cannot use --pre-release-label with --no-pre-release")] // label + no_pre_release
//         #[case(
//             None,
//             Some(5),
//             true,
//             "Cannot use --pre-release-num with --no-pre-release"
//         )] // num + no_pre_release
//         #[case(Some("rc".to_string()), Some(10), true, "Cannot use --pre-release-label with --no-pre-release")] // both + no_pre_release (first error)
//         fn test_no_pre_release_conflicts(
//             #[case] label: Option<String>,
//             #[case] num: Option<u32>,
//             #[case] no_pre_release: bool,
//             #[case] expected_error: &str,
//         ) {
//             let mut args = FlowArgs {
//                 pre_release_label: label,
//                 pre_release_num: num,
//                 no_pre_release,
//                 ..FlowArgs::default()
//             };
//             let result = args.validate();
//             assert!(result.is_err());
//             assert!(result.unwrap_err().to_string().contains(expected_error));
//         }

//         #[rstest]
//         #[case(Some("beta".to_string()), Some(5), false)]
//         #[case(None, None, true)]
//         #[case(Some("rc".to_string()), None, false)]
//         #[case(None, Some(10), false)]
//         fn test_valid_combinations(
//             #[case] label: Option<String>,
//             #[case] num: Option<u32>,
//             #[case] no_pre_release: bool,
//         ) {
//             let mut args = FlowArgs {
//                 pre_release_label: label,
//                 pre_release_num: num,
//                 no_pre_release,
//                 ..FlowArgs::default()
//             };
//             assert!(args.validate().is_ok());
//         }
//     }

//     mod input_output {
//         use super::*;

//         #[test]
//         fn test_flow_args_with_custom_input_output() {
//             let mut args = FlowArgs {
//                 input: InputConfig {
//                     source: "git".to_string(),
//                     input_format: "auto".to_string(),
//                     directory: Some("/test/path".to_string()),
//                 },
//                 output: OutputConfig {
//                     output_format: "zerv".to_string(),
//                     output_prefix: Some("v".to_string()),
//                     output_template: None,
//                 },
//                 ..FlowArgs::default()
//             };
//             assert_eq!(args.input.source, "git");
//             assert_eq!(args.output.output_format, "zerv");
//             assert_eq!(args.output.output_prefix, Some("v".to_string()));
//             assert!(args.validate().is_ok());
//         }
//     }

//     mod bump_pre_release_label {
//         use super::*;

//         #[test]
//         fn test_default_returns_alpha() {
//             let mut args = FlowArgs::default();
//             args.validate().unwrap(); // This sets the default pre_release_label
//             assert_eq!(args.bump_pre_release_label(), Some("alpha".to_string()));
//         }

//         #[rstest]
//         #[case("beta")]
//         #[case("rc")]
//         fn test_custom_label_returned(#[case] label: &str) {
//             let args = FlowArgs {
//                 pre_release_label: Some(label.to_string()),
//                 ..FlowArgs::default()
//             };
//             assert_eq!(args.bump_pre_release_label(), Some(label.to_string()));
//         }

//         #[test]
//         fn test_default_alpha_when_no_pre_release_false_and_label_none() {
//             // Test the specific case: no_pre_release=false AND pre_release_label=None => returns alpha
//             let mut args = FlowArgs {
//                 no_pre_release: false,
//                 pre_release_label: None,
//                 ..FlowArgs::default()
//             };
//             // Before validation, pre_release_label should be None
//             assert_eq!(args.pre_release_label, None);

//             args.validate().unwrap(); // This sets the default pre_release_label

//             // After validation, pre_release_label should be set to "alpha"
//             assert_eq!(args.pre_release_label, Some("alpha".to_string()));
//             assert_eq!(args.bump_pre_release_label(), Some("alpha".to_string()));
//         }

//         #[test]
//         fn test_disabled_returns_none() {
//             let args = FlowArgs {
//                 no_pre_release: true,
//                 ..FlowArgs::default()
//             };
//             assert_eq!(args.bump_pre_release_label(), None);
//         }
//     }

//     mod bump_pre_release_num {
//         use super::*;

//         #[test]
//         fn test_default_returns_template() {
//             let args = FlowArgs::default();
//             let result = args.bump_pre_release_num();
//             assert!(result.is_some());
//             assert!(result.unwrap().is_some()); // Should be Some(Template)
//         }

//         #[rstest]
//         #[case(5)]
//         #[case(123)]
//         #[case(999)]
//         fn test_custom_num_returns_value(#[case] num: u32) {
//             let args = FlowArgs {
//                 pre_release_num: Some(num),
//                 ..FlowArgs::default()
//             };
//             let result = args.bump_pre_release_num();
//             assert!(result.is_some());
//             match result.unwrap().unwrap() {
//                 Template::Value(value) => assert_eq!(value, num),
//                 _ => panic!("Expected Template::Value"),
//             }
//         }

//         #[test]
//         fn test_disabled_returns_none() {
//             let args = FlowArgs {
//                 no_pre_release: true,
//                 ..FlowArgs::default()
//             };
//             assert_eq!(args.bump_pre_release_num(), None);
//         }

//         #[rstest]
//         #[case(3, "{{hash_int bumped_branch 3}}")]
//         #[case(7, "{{hash_int bumped_branch 7}}")]
//         #[case(5, "{{hash_int bumped_branch 5}}")]
//         fn test_template_uses_hash_branch_len(#[case] length: u32, #[case] expected: &str) {
//             let args = FlowArgs {
//                 hash_branch_len: length,
//                 ..FlowArgs::default()
//             };
//             let result = args.bump_pre_release_num();
//             assert!(result.is_some());
//             match result.unwrap().unwrap() {
//                 Template::Template(template) => assert_eq!(template, expected),
//                 _ => panic!("Expected Template::Template"),
//             }
//         }
//     }

//     mod error_cases {
//         use super::*;

//         #[rstest]
//         #[case("invalid")]
//         #[case("dev")]
//         #[case("alpha1")]
//         #[case("alpha-beta")]
//         #[case("ALPHA")]
//         #[case("")]
//         fn test_invalid_pre_release_label_choices(#[case] invalid_label: &str) {
//             // Note: clap's PossibleValuesParser handles validation at parsing time
//             // This test demonstrates what happens when invalid values are somehow set
//             let mut args = FlowArgs {
//                 pre_release_label: Some(invalid_label.to_string()),
//                 ..FlowArgs::default()
//             };

//             // Note: Our current validation only checks structural constraints, not value constraints
//             // The clap argument parser would catch invalid values before they reach validate()
//             // This test shows the validation passes but the values would be rejected by clap
//             assert!(args.validate().is_ok());
//         }
//     }

//     mod integration {
//         use super::*;

//         #[rstest]
//         #[case("alpha", Some(5), 3)]
//         #[case("beta", None, 7)]
//         #[case("rc", Some(10), 1)]
//         fn test_complete_configuration(
//             #[case] label: &str,
//             #[case] num: Option<u32>,
//             #[case] hash_len: u32,
//         ) {
//             let mut args = FlowArgs {
//                 pre_release_label: Some(label.to_string()),
//                 pre_release_num: num,
//                 hash_branch_len: hash_len,
//                 ..FlowArgs::default()
//             };

//             // Test validation
//             assert!(args.validate().is_ok());

//             // Test bump_pre_release_label
//             assert_eq!(args.bump_pre_release_label(), Some(label.to_string()));

//             // Test bump_pre_release_num
//             let result = args.bump_pre_release_num();
//             assert!(result.is_some());

//             if let Some(num_value) = num {
//                 match result.unwrap().unwrap() {
//                     Template::Value(value) => assert_eq!(value, num_value),
//                     _ => panic!("Expected Template::Value when pre_release_num is set"),
//                 }
//             } else {
//                 match result.unwrap().unwrap() {
//                     Template::Template(template) => {
//                         assert_eq!(
//                             template,
//                             format!("{{{{hash_int bumped_branch {}}}}}", hash_len)
//                         );
//                     }
//                     _ => panic!("Expected Template::Template when pre_release_num is not set"),
//                 }
//             }
//         }
//     }
// }
