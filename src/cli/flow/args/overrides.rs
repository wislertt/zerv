use clap::Parser;

use crate::utils::bool_resolution::BoolResolution;

/// Override configuration for VCS and version components (same as zerv version)
#[derive(Parser, Default, Debug)]
pub struct OverridesConfig {
    // ============================================================================
    // VCS OVERRIDE OPTIONS (same as zerv version)
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
    #[arg(long, help = "Override current branch name for pattern matching")]
    pub current_branch: Option<String>,

    /// Override the detected commit hash
    #[arg(long, help = "Override commit hash (full or short form)")]
    pub commit_hash: Option<String>,
}

impl OverridesConfig {
    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    pub fn dirty_override(&self) -> Option<bool> {
        BoolResolution::resolve_opposing_flags(self.dirty, self.no_dirty)
    }
}
