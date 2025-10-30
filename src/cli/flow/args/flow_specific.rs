use clap::Parser;

use crate::utils::constants::post_modes;

/// Flow-specific configuration for branch pattern detection and output modes
#[derive(Parser, Debug)]
pub struct FlowSpecificConfig {
    // ============================================================================
    // FLOW-SPECIFIC OPTIONS
    // ============================================================================
    /// Custom branch pattern rules in RON format
    #[arg(long = "branch-rules", value_name = "RON")]
    pub branch_rules: Option<String>,

    /// Show pre-release information in output (default)
    #[arg(long = "with-pre-release")]
    pub with_pre_release: bool,

    /// Show only base version without pre-release
    #[arg(long = "base-only")]
    pub base_only: bool,

    /// Post distance calculation mode
    #[arg(long = "post-mode", value_name = "MODE", default_value = post_modes::TAG,
          help = "Post distance calculation mode: 'tag' (count from last tag) or 'commit' (count from branch creation)")]
    pub post_mode: String,
}

impl Default for FlowSpecificConfig {
    fn default() -> Self {
        Self {
            branch_rules: None,
            with_pre_release: false,
            base_only: false,
            post_mode: post_modes::TAG.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_specific_config_default() {
        let config = FlowSpecificConfig::default();
        assert_eq!(config.branch_rules, None);
        assert!(!config.with_pre_release);
        assert!(!config.base_only);
        assert_eq!(config.post_mode, post_modes::TAG);
    }

    #[test]
    fn test_flow_specific_config_with_pre_release_true() {
        let config = FlowSpecificConfig {
            with_pre_release: true,
            base_only: false,
            ..Default::default()
        };
        assert!(config.with_pre_release);
        assert!(!config.base_only);
    }

    #[test]
    fn test_flow_specific_config_base_only_true() {
        let config = FlowSpecificConfig {
            with_pre_release: false,
            base_only: true,
            ..Default::default()
        };
        assert!(!config.with_pre_release);
        assert!(config.base_only);
    }

    #[test]
    fn test_flow_specific_config_post_mode_tag() {
        let config = FlowSpecificConfig {
            post_mode: post_modes::TAG.to_string(),
            ..Default::default()
        };
        assert_eq!(config.post_mode, post_modes::TAG);
    }

    #[test]
    fn test_flow_specific_config_post_mode_commit() {
        let config = FlowSpecificConfig {
            post_mode: post_modes::COMMIT.to_string(),
            ..Default::default()
        };
        assert_eq!(config.post_mode, post_modes::COMMIT);
    }
}
