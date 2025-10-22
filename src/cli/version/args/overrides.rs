use clap::Parser;

use crate::cli::utils::template::Template;
use crate::utils::constants::pre_release_labels;

/// Override configuration for VCS and version components
#[derive(Parser, Default)]
pub struct OverridesConfig {
    // ============================================================================
    // VCS OVERRIDE OPTIONS
    // ============================================================================
    /// Override the detected tag version
    #[arg(
        long,
        help = "Override detected tag version (e.g., 'v2.0.0', '1.5.0-beta.1')"
    )]
    pub tag_version: Option<String>,

    /// Override the calculated distance from tag
    #[arg(
        long,
        help = "Override distance from tag (number of commits since tag)"
    )]
    pub distance: Option<u32>,

    /// Override the detected dirty state (sets dirty=true)
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Override dirty state to true (sets dirty=true)")]
    pub dirty: bool,

    /// Override the detected dirty state (sets dirty=false)
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Override dirty state to false (sets dirty=false)")]
    pub no_dirty: bool,

    /// Set distance=None and dirty=false (clean release state)
    #[arg(
        long,
        help = "Force clean release state (sets distance=0, dirty=false). Conflicts with --distance and --dirty"
    )]
    pub clean: bool,

    /// Override the detected current branch name
    #[arg(long, help = "Override current branch name")]
    pub bumped_branch: Option<String>,

    /// Override the detected commit hash
    #[arg(long, help = "Override commit hash (full or short form)")]
    pub bumped_commit_hash: Option<String>,

    /// Override the detected commit timestamp
    #[arg(long, help = "Override commit timestamp (Unix timestamp)")]
    pub bumped_timestamp: Option<i64>,

    // ============================================================================
    // VERSION COMPONENT OVERRIDE OPTIONS
    // ============================================================================
    /// Override major version number
    #[arg(long, help = "Override major version number")]
    pub major: Option<Template<u32>>,

    /// Override minor version number
    #[arg(long, help = "Override minor version number")]
    pub minor: Option<Template<u32>>,

    /// Override patch version number
    #[arg(long, help = "Override patch version number")]
    pub patch: Option<Template<u32>>,

    /// Override epoch number
    #[arg(long, help = "Override epoch number")]
    pub epoch: Option<Template<u32>>,

    /// Override post number
    #[arg(long, help = "Override post number")]
    pub post: Option<Template<u32>>,

    /// Override dev number
    #[arg(long, help = "Override dev number")]
    pub dev: Option<Template<u32>>,

    /// Override pre-release label
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(pre_release_labels::VALID_LABELS),
          help = "Override pre-release label (alpha, beta, rc)")]
    pub pre_release_label: Option<String>,

    /// Override pre-release number
    #[arg(long, help = "Override pre-release number")]
    pub pre_release_num: Option<Template<u32>>,

    /// Override custom variables in JSON format
    #[arg(long, help = "Override custom variables in JSON format")]
    pub custom: Option<String>,

    // ============================================================================
    // SCHEMA COMPONENT OVERRIDE OPTIONS
    // ============================================================================
    /// Override core schema component by index=value
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override core schema component by index=value (e.g., --core 0=5 or --core 1={{major}})"
    )]
    pub core: Vec<Template<String>>,

    /// Override extra-core schema component by index=value
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override extra-core schema component by index=value (e.g., --extra-core 0=5 or --extra-core 1={{branch}})"
    )]
    pub extra_core: Vec<Template<String>>,

    /// Override build schema component by index=value
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override build schema component by index=value (e.g., --build 0=5 or --build 1={{commit_short}})"
    )]
    pub build: Vec<Template<String>>,
}

impl OverridesConfig {
    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    pub fn dirty_override(&self) -> Option<bool> {
        match (self.dirty, self.no_dirty) {
            (true, false) => Some(true),    // --dirty
            (false, true) => Some(false),   // --no-dirty
            (false, false) => None,         // neither (use VCS)
            (true, true) => unreachable!(), // Should be caught by validation
        }
    }
}
