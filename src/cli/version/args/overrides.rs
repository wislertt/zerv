use clap::Parser;

use crate::cli::common::overrides::CommonOverridesConfig;
use crate::cli::utils::template::Template;

/// Override configuration for version command
#[derive(Parser, Default, Debug)]
pub struct OverridesConfig {
    #[command(flatten)]
    pub common: CommonOverridesConfig,

    // ============================================================================
    // VERSION-SPECIFIC OVERRIDE OPTIONS
    // ============================================================================
    /// Override dev number
    #[arg(long, help = "Override dev number")]
    pub dev: Option<Template<u32>>,

    /// Override pre-release label
    #[arg(
        long,
        help = "Override pre-release label (alpha, beta, rc, none, null). Supports templates like '{{{{#if dirty}}}}dev{{{{else}}}}beta{{{{/if}}}}'"
    )]
    pub pre_release_label: Option<Template<String>>,

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
        help = "Override core schema component by index=value (e.g., --core 0=5, --core ~1=2024, --core 1={{major}})"
    )]
    pub core: Vec<Template<String>>,

    /// Override extra-core schema component by index=value
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override extra-core schema component by index=value (e.g., --extra-core 0=5, --extra-core ~1=beta, --extra-core 1={{branch}})"
    )]
    pub extra_core: Vec<Template<String>>,

    /// Override build schema component by index=value
    #[arg(
        long,
        value_name = "INDEX=VALUE",
        num_args = 1..,
        help = "Override build schema component by index=value (e.g., --build 0=5, --build ~1=release, --build 1={{commit_short}})"
    )]
    pub build: Vec<Template<String>>,
}

impl OverridesConfig {
    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    pub fn dirty_override(&self) -> Option<bool> {
        self.common.dirty_override()
    }
}
