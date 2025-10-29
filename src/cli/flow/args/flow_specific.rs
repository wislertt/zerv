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
