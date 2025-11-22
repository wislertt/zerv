use clap::Parser;

use crate::cli::utils::template::Template;

/// Override configuration for VCS and version components
#[derive(Parser, Default, Debug)]
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
}

impl OverridesConfig {
    /// Get post override value or default template
    pub fn override_post(&self) -> Option<Template<u32>> {
        self.post
            .clone()
            .or_else(|| Some(Template::new("{{ post }}".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod defaults {
        use super::*;

        #[test]
        fn test_overrides_config_default() {
            let config = OverridesConfig::default();
            assert!(config.tag_version.is_none());
            assert!(config.distance.is_none());
            assert!(!config.dirty);
            assert!(!config.no_dirty);
            assert!(!config.clean);
            assert!(config.bumped_branch.is_none());
            assert!(config.bumped_commit_hash.is_none());
            assert!(config.bumped_timestamp.is_none());
            assert!(config.major.is_none());
            assert!(config.minor.is_none());
            assert!(config.patch.is_none());
            assert!(config.epoch.is_none());
            assert!(config.post.is_none());
            assert!(config.dev.is_none());
        }
    }
}
