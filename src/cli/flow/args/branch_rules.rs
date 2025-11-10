use clap::Parser;

use crate::cli::flow::branch_rules::BranchRules;
use crate::error::ZervError;
use crate::utils::constants::post_modes;
use crate::version::zerv::core::Zerv;

/// Configuration for branch-related settings
#[derive(Parser, Debug, Clone)]
pub struct BranchRulesConfig {
    /// Pre-release label for flow versions (alpha, beta, rc)
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(crate::utils::constants::pre_release_labels::VALID_LABELS),
          help = "Pre-release label for flow versions (alpha, beta, rc)")]
    pub pre_release_label: Option<String>,

    #[arg(
        long,
        value_parser = clap::value_parser!(u32),
        help = "Pre-release number for flow versions (integer, default: {{ hash_int(value=bumped_branch, length=HASH_BRANCH_LEN) }})"
    )]
    pub pre_release_num: Option<u32>,

    /// Post calculation mode (commit, tag)
    #[arg(long = "post-mode", value_parser = clap::builder::PossibleValuesParser::new(post_modes::VALID_MODES),
          help = "Post calculation mode (commit, tag)")]
    pub post_mode: Option<String>,

    /// Branch rules in RON format (default: GitFlow rules)
    #[arg(
        long = "branch-rules",
        help = "Branch rules in RON format (default: GitFlow rules)",
        value_parser = clap::value_parser!(BranchRules),
        default_value_t = BranchRules::default_rules(),
    )]
    pub branch_rules: BranchRules,
}

impl Default for BranchRulesConfig {
    fn default() -> Self {
        Self {
            pre_release_label: None,
            pre_release_num: None,
            post_mode: None,
            branch_rules: BranchRules::default_rules(),
        }
    }
}

impl BranchRulesConfig {
    /// Check if any branch configuration is explicitly set
    pub fn has_explicit_settings(&self) -> bool {
        self.pre_release_label.is_some()
            || self.pre_release_num.is_some()
            || self.post_mode.is_some()
    }

    /// Apply branch rules using provided zerv object
    pub fn apply_branch_rules(&mut self, current_zerv: &Zerv) -> Result<(), ZervError> {
        let resolved_args = self
            .branch_rules
            .resolve_for_branch(current_zerv.vars.bumped_branch.as_deref());

        if self.pre_release_label.is_none() {
            self.pre_release_label = Some(resolved_args.pre_release_label.to_string().into());
        }
        if self.pre_release_num.is_none() {
            self.pre_release_num = resolved_args.pre_release_num;
        }
        if self.post_mode.is_none() {
            self.post_mode = Some(resolved_args.post_mode.to_string().into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[test]
    fn test_branch_rules_config_default() {
        let config = BranchRulesConfig::default();
        assert!(config.pre_release_label.is_none());
        assert!(config.pre_release_num.is_none());
        assert!(config.post_mode.is_none());
        assert!(!config.has_explicit_settings());
    }

    #[test]
    fn test_branch_rules_config_has_explicit_settings() {
        let config = BranchRulesConfig {
            pre_release_label: Some("alpha".to_string()),
            pre_release_num: None,
            post_mode: None,
            branch_rules: BranchRules::default_rules(),
        };
        assert!(config.has_explicit_settings());
    }

    #[test]
    fn test_branch_rules_config_no_explicit_settings() {
        let config = BranchRulesConfig {
            pre_release_label: None,
            pre_release_num: None,
            post_mode: None,
            branch_rules: BranchRules::default_rules(),
        };
        assert!(!config.has_explicit_settings());
    }

    #[rstest]
    #[case("alpha")]
    #[case("beta")]
    #[case("rc")]
    fn test_valid_pre_release_labels(#[case] label: &str) {
        let config = BranchRulesConfig {
            pre_release_label: Some(label.to_string()),
            ..BranchRulesConfig::default()
        };
        assert_eq!(config.pre_release_label, Some(label.to_string()));
    }

    #[rstest]
    #[case(1)]
    #[case(42)]
    #[case(999)]
    fn test_valid_pre_release_nums(#[case] num: u32) {
        let config = BranchRulesConfig {
            pre_release_num: Some(num),
            ..BranchRulesConfig::default()
        };
        assert_eq!(config.pre_release_num, Some(num));
    }

    #[rstest]
    #[case("commit")]
    #[case("tag")]
    fn test_valid_post_modes(#[case] mode: &str) {
        let config = BranchRulesConfig {
            post_mode: Some(mode.to_string()),
            ..BranchRulesConfig::default()
        };
        assert_eq!(config.post_mode, Some(mode.to_string()));
    }

    mod branch_rules {
        use super::*;
        use crate::cli::flow::args::main::FlowArgs;
        use crate::test_utils::zerv::ZervFixture;
        use crate::version::zerv::core::Zerv;

        /// Helper function to create a mock zerv object for tests
        fn mock_zerv() -> Zerv {
            let mut zerv = ZervFixture::new().build();
            // Set a mock branch name for tests that need branch detection
            zerv.vars.last_branch = Some("main".to_string());
            zerv
        }

        #[test]
        fn test_flow_args_default_has_gitflow_rules() {
            let args = FlowArgs::default();
            // Should have default GitFlow rules
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

        #[test]
        fn test_flow_args_with_custom_branch_rules() {
            let custom_ron = r#"[
                (pattern: "main", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
                (pattern: "hotfix/*", pre_release_label: rc, post_mode: tag)
            ]"#;

            let custom_rules: BranchRules = custom_ron.parse().unwrap();
            let args = FlowArgs {
                branch_config: BranchRulesConfig {
                    branch_rules: custom_rules,
                    ..Default::default()
                },
                ..FlowArgs::default()
            };

            // Should have custom rules, not default GitFlow rules
            assert!(args.branch_config.branch_rules.find_rule("main").is_some());
            assert!(
                args.branch_config
                    .branch_rules
                    .find_rule("hotfix/123")
                    .is_some()
            );
            assert!(
                args.branch_config
                    .branch_rules
                    .find_rule("develop")
                    .is_none()
            ); // Not in custom rules
            assert!(
                args.branch_config
                    .branch_rules
                    .find_rule("release/1")
                    .is_none()
            ); // Not in custom rules
        }

        #[test]
        fn test_flow_args_validation_with_custom_branch_rules() {
            let custom_ron = r#"[
                (pattern: "develop", pre_release_label: beta, pre_release_num: 1, post_mode: commit),
                (pattern: "release/*", pre_release_label: rc, post_mode: tag)
            ]"#;

            let mut args = FlowArgs {
                branch_config: BranchRulesConfig {
                    branch_rules: custom_ron.parse().unwrap(),
                    ..Default::default()
                },
                ..FlowArgs::default()
            };

            // Validation should succeed even with custom branch rules
            assert!(args.validate(&mock_zerv()).is_ok());
        }
    }
}
