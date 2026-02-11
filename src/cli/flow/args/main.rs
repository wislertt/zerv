use clap::Parser;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
};
use crate::cli::flow::args::branch_rules::BranchRulesConfig;
use crate::cli::flow::args::overrides::OverridesConfig;

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
  --schema <SCHEMA>         Schema variant for output components [default: standard]

Standard Schema Family (SemVer):
  standard                        - Smart auto-detection based on repository state (clean/dirty/distance)
  standard-base                   - 1.1.0
  standard-base-prerelease        - 1.1.0-alpha.1
  standard-base-prerelease-post   - 1.1.0-alpha.1.post.2
  standard-base-prerelease-post-dev - 1.1.0-alpha.1.post.2.dev.1729924622
  standard-base-context           - 1.1.0+main.2.a1b2c3d
  standard-base-prerelease-context - 1.1.0-alpha.1+main.2.a1b2c3d
  standard-base-prerelease-post-context - 1.1.0-alpha.1.post.2+main.2.a1b2c3d
  standard-base-prerelease-post-dev-context - 1.1.0-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
  standard-context                - Smart auto-detection with build context

CalVer Schema Family:
  calver                          - Smart auto-detection based on repository state (clean/dirty/distance)
  calver-base                     - 2024.11.03
  calver-base-prerelease          - 2024.11.03-alpha.1
  calver-base-prerelease-post     - 2024.11.03-alpha.1.post.2
  calver-base-prerelease-post-dev - 2024.11.03-alpha.1.post.2.dev.1729924622
  calver-base-context             - 2024.11.03+main.2.a1b2c3d
  calver-base-prerelease-context  - 2024.11.03-alpha.1+main.2.a1b2c3d
  calver-base-prerelease-post-context - 2024.11.03-alpha.1.post.2+main.2.a1b2c3d
  calver-base-prerelease-post-dev-context - 2024.11.03-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
  calver-context                  - Smart auto-detection with build context

VCS OVERRIDE OPTIONS:
  --tag-version <VERSION>   Override detected tag version (e.g., 'v2.0.0', '1.5.0-beta.1')
  --distance <NUM>          Override distance from tag (number of commits since tag)
  --dirty/--no-dirty        Override dirty state to true/false
  --clean                   Force clean release state (sets distance=0, dirty=false)
  --bumped-branch <BRANCH>  Override current branch name
  --bumped-commit-hash <HASH> Override commit hash (full or short form)
  --bumped-timestamp <TS>   Override commit timestamp (Unix timestamp)

VERSION COMPONENT OVERRIDES:
  --major <NUM>             Override major version number
  --minor <NUM>             Override minor version number
  --patch <NUM>             Override patch version number
  --epoch <NUM>             Override epoch number
  --post <NUM>              Override post number
  --dev <NUM>               Override dev number

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

  # Override VCS values for testing or CI/CD
  zerv flow --tag-version v2.0.0 --distance 5 --dirty
  zerv flow --clean  # Force clean release state
  zerv flow --bumped-branch feature/test  # Override branch name

  # Override version components
  zerv flow --major 2 --minor 0 --patch 0
  zerv flow --post 1 --dev 1234567890

  # Combined overrides and branch rules
  zerv flow --schema standard-base --tag-version v1.5.0 --pre-release-label beta

  # Use in different directory
  zerv flow -C /path/to/repo"
)]
#[derive(Debug)]
pub struct FlowArgs {
    #[command(flatten)]
    pub input: InputConfig,

    #[command(flatten)]
    pub output: OutputConfig,

    #[command(flatten)]
    pub branch_config: BranchRulesConfig,

    #[command(flatten)]
    pub overrides: OverridesConfig,

    #[arg(
        long = "hash-branch-len",
        value_parser = clap::value_parser!(u32),
        default_value = "5",
        help = "Hash length for bumped branch hash (1-10, default: 5)"
    )]
    pub hash_branch_len: u32,

    /// Schema preset name
    #[arg(
        long,
        help = "Schema preset name

Standard Schema Family (SemVer):
  standard                        - Smart auto-detection based on repository state (clean/dirty/distance)
  standard-base                   - 1.1.0
  standard-base-prerelease        - 1.1.0-alpha.1
  standard-base-prerelease-post   - 1.1.0-alpha.1.post.2
  standard-base-prerelease-post-dev - 1.1.0-alpha.1.post.2.dev.1729924622
  standard-base-context           - 1.1.0+main.2.a1b2c3d
  standard-base-prerelease-context - 1.1.0-alpha.1+main.2.a1b2c3d
  standard-base-prerelease-post-context - 1.1.0-alpha.1.post.2+main.2.a1b2c3d
  standard-base-prerelease-post-dev-context - 1.1.0-alpha.1.post.2.dev.1729924622+main.2.a1b2c3d
  standard-context                - Smart auto-detection with build context
"
    )]
    pub schema: Option<String>,

    /// Custom RON schema definition
    #[arg(long, help = "Custom schema in RON format")]
    pub schema_ron: Option<String>,
}

impl Default for FlowArgs {
    fn default() -> Self {
        Self {
            input: InputConfig::default(),
            output: OutputConfig::default(),
            branch_config: BranchRulesConfig::default(),
            overrides: OverridesConfig::default(),
            hash_branch_len: 5,
            schema: None,
            schema_ron: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::cli::flow::args::branch_rules::BranchRulesConfig;
    use crate::test_utils::zerv::ZervFixture;
    use crate::version::zerv::core::Zerv;

    /// Helper function to create a mock zerv object for tests
    fn mock_zerv() -> Zerv {
        let mut zerv = ZervFixture::new().build();
        // Set a mock branch name for tests that need branch detection
        zerv.vars.last_branch = Some("main".to_string());
        zerv
    }

    mod defaults {
        use super::*;

        #[test]
        fn test_flow_args_default() {
            let args = FlowArgs::default();
            assert_eq!(args.input.source, Some("git".to_string()));
            assert_eq!(args.output.output_format, "semver");
            assert_eq!(args.hash_branch_len, 5);
            assert!(args.branch_config.pre_release_label.is_none());
            assert!(args.branch_config.pre_release_num.is_none());
            assert_eq!(args.branch_config.post_mode, None);
            assert!(args.schema.is_none()); // Default is None (will use standard schema)
            assert!(args.schema_ron.is_none()); // Default is None
            assert!(args.overrides.common.bumped_branch.is_none()); // Default is None (use detected branch)
        }

        #[test]
        fn test_flow_args_validation_success() {
            let mut args = FlowArgs::default();
            assert!(args.validate(&mock_zerv(), None).is_ok());
            assert!(args.schema.is_none()); // Should remain None after validation
        }
    }

    mod input_output {
        use super::*;

        #[test]
        fn test_flow_args_with_custom_input_output() {
            let mut args = FlowArgs {
                input: InputConfig {
                    source: Some("git".to_string()),
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
            assert_eq!(args.input.source, Some("git".to_string()));
            assert_eq!(args.output.output_format, "zerv");
            assert_eq!(args.output.output_prefix, Some("v".to_string()));
            assert!(args.validate(&mock_zerv(), None).is_ok());
        }

        #[test]
        fn test_flow_args_with_schema_ron() {
            let ron_schema = "core: [{var: \"major\"}]";
            let mut args = FlowArgs {
                schema: None,
                schema_ron: Some(ron_schema.to_string()),
                ..FlowArgs::default()
            };
            assert!(args.schema.is_none());
            assert_eq!(args.schema_ron, Some(ron_schema.to_string()));
            assert!(args.validate(&mock_zerv(), None).is_ok());
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
                branch_config: BranchRulesConfig {
                    pre_release_label: Some(invalid_label.to_string()),
                    ..Default::default()
                },
                ..FlowArgs::default()
            };

            // Note: Our current validation only checks structural constraints, not value constraints
            // The clap argument parser would catch invalid values before they reach validate()
            // This test shows the validation passes but the values would be rejected by clap
            assert!(args.validate(&mock_zerv(), None).is_ok());
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn test_integration_with_branch_rules_and_manual_overrides() {
            let custom_ron = r#"[
                (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
                (pattern: "release/*", pre_release_label: rc, post_mode: tag)
            ]"#;

            let mut args = FlowArgs {
                branch_config: BranchRulesConfig {
                    branch_rules: custom_ron.parse().unwrap(),
                    pre_release_label: Some("alpha".to_string()), // Manual override
                    pre_release_num: Some(42),                    // Manual override
                    post_mode: Some("tag".to_string()),           // Manual override
                },
                ..FlowArgs::default()
            };

            // Should validate successfully - branch rules and manual overrides can coexist
            assert!(args.validate(&mock_zerv(), None).is_ok());

            // Manual overrides should be preserved
            assert_eq!(
                args.branch_config.pre_release_label,
                Some("alpha".to_string())
            );
            assert_eq!(args.branch_config.pre_release_num, Some(42));
            assert_eq!(args.branch_config.post_mode, Some("tag".to_string()));

            // Branch rules should still be available for validation logic
            assert!(
                args.branch_config
                    .branch_rules
                    .find_rule("develop")
                    .is_some()
            );
            assert!(
                args.branch_config
                    .branch_rules
                    .find_rule("release/1")
                    .is_some()
            );
        }
    }
}
