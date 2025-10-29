use clap::Parser;
use clap::builder::PossibleValuesParser;

use crate::cli::utils::template::Template;
use crate::utils::constants::pre_release_labels;

/// Bump configuration for field-based and schema-based version bumping
#[derive(Parser, Default, Debug)]
pub struct BumpsConfig {
    // ============================================================================
    // FIELD-BASED BUMP OPTIONS
    // ============================================================================
    /// Add to major version (default: 1)
    #[arg(long, help = "Add to major version (default: 1)")]
    pub bump_major: Option<Option<Template<u32>>>,

    /// Add to minor version (default: 1)
    #[arg(long, help = "Add to minor version (default: 1)")]
    pub bump_minor: Option<Option<Template<u32>>>,

    /// Add to patch version (default: 1)
    #[arg(long, help = "Add to patch version (default: 1)")]
    pub bump_patch: Option<Option<Template<u32>>>,

    /// Add to post number (default: 1)
    #[arg(long, help = "Add to post number (default: 1)")]
    pub bump_post: Option<Option<Template<u32>>>,

    /// Add to dev number (default: 1)
    #[arg(long, help = "Add to dev number (default: 1)")]
    pub bump_dev: Option<Option<Template<u32>>>,

    /// Add to pre-release number (default: 1)
    #[arg(long, help = "Add to pre-release number (default: 1)")]
    pub bump_pre_release_num: Option<Option<Template<u32>>>,

    /// Add to epoch number (default: 1)
    #[arg(long, help = "Add to epoch number (default: 1)")]
    pub bump_epoch: Option<Option<Template<u32>>>,

    /// Bump pre-release label (alpha, beta, rc) and reset number to 0
    #[arg(long, value_parser = PossibleValuesParser::new(pre_release_labels::VALID_LABELS),
          help = "Bump pre-release label (alpha, beta, rc) and reset number to 0")]
    pub bump_pre_release_label: Option<String>,

    // ============================================================================
    // SCHEMA-BASED BUMP OPTIONS
    // ============================================================================
    /// Bump core schema component by index[=value] (default value: 1)
    #[arg(
        long,
        value_name = "INDEX[=VALUE]",
        num_args = 1..,
        help = "Bump core schema component by index[=value] (e.g., --bump-core 0={{distance}} or --bump-core 0)"
    )]
    pub bump_core: Vec<Template<String>>,

    /// Bump extra-core schema component by index[=value] (default value: 1)
    #[arg(
        long,
        value_name = "INDEX[=VALUE]",
        num_args = 1..,
        help = "Bump extra-core schema component by index[=value] (e.g., --bump-extra-core 0={{distance}} or --bump-extra-core 0)"
    )]
    pub bump_extra_core: Vec<Template<String>>,

    /// Bump build schema component by index[=value] (default value: 1)
    #[arg(
        long,
        value_name = "INDEX[=VALUE]",
        num_args = 1..,
        help = "Bump build schema component by index[=value] (e.g., --bump-build 0={{distance}} or --bump-build 0)"
    )]
    pub bump_build: Vec<Template<String>>,

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
