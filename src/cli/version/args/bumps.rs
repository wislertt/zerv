use clap::Parser;

use crate::constants::pre_release_labels;

/// Bump configuration for field-based and schema-based version bumping
#[derive(Parser, Default)]
pub struct BumpsConfig {
    // ============================================================================
    // FIELD-BASED BUMP OPTIONS
    // ============================================================================
    /// Add to major version (default: 1)
    #[arg(long, help = "Add to major version (default: 1)")]
    pub bump_major: Option<Option<u32>>,

    /// Add to minor version (default: 1)
    #[arg(long, help = "Add to minor version (default: 1)")]
    pub bump_minor: Option<Option<u32>>,

    /// Add to patch version (default: 1)
    #[arg(long, help = "Add to patch version (default: 1)")]
    pub bump_patch: Option<Option<u32>>,

    /// Add to post number (default: 1)
    #[arg(long, help = "Add to post number (default: 1)")]
    pub bump_post: Option<Option<u32>>,

    /// Add to dev number (default: 1)
    #[arg(long, help = "Add to dev number (default: 1)")]
    pub bump_dev: Option<Option<u32>>,

    /// Add to pre-release number (default: 1)
    #[arg(long, help = "Add to pre-release number (default: 1)")]
    pub bump_pre_release_num: Option<Option<u32>>,

    /// Add to epoch number (default: 1)
    #[arg(long, help = "Add to epoch number (default: 1)")]
    pub bump_epoch: Option<Option<u32>>,

    /// Bump pre-release label (alpha, beta, rc) and reset number to 0
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(pre_release_labels::VALID_LABELS),
          help = "Bump pre-release label (alpha, beta, rc) and reset number to 0")]
    pub bump_pre_release_label: Option<String>,

    // ============================================================================
    // SCHEMA-BASED BUMP OPTIONS
    // ============================================================================
    /// Bump core schema component by index and value
    #[arg(
        long,
        value_name = "INDEX VALUE",
        num_args = 2,
        help = "Bump core schema component by index and value (pairs of index, value)"
    )]
    pub bump_core: Vec<u32>,

    /// Bump extra-core schema component by index and value
    #[arg(
        long,
        value_name = "INDEX VALUE",
        num_args = 2,
        help = "Bump extra-core schema component by index and value (pairs of index, value)"
    )]
    pub bump_extra_core: Vec<u32>,

    /// Bump build schema component by index and value
    #[arg(
        long,
        value_name = "INDEX VALUE",
        num_args = 2,
        help = "Bump build schema component by index and value (pairs of index, value)"
    )]
    pub bump_build: Vec<u32>,

    // ============================================================================
    // CONTEXT CONTROL OPTIONS
    // ============================================================================
    /// Include VCS context qualifiers (default behavior)
    #[arg(long, help = "Include VCS context qualifiers (default behavior)")]
    pub bump_context: bool,

    /// Pure tag version, no VCS context
    #[arg(long, help = "Pure tag version, no VCS context")]
    pub no_bump_context: bool,
}
