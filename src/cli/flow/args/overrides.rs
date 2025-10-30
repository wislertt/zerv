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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overrides_config_default() {
        let config = OverridesConfig::default();
        assert_eq!(config.tag_version, None);
        assert_eq!(config.distance, None);
        assert!(!config.dirty);
        assert!(!config.no_dirty);
        assert!(!config.clean);
        assert_eq!(config.current_branch, None);
        assert_eq!(config.commit_hash, None);
    }

    #[test]
    fn test_dirty_override_true() {
        let config = OverridesConfig {
            dirty: true,
            no_dirty: false,
            ..Default::default()
        };
        assert_eq!(config.dirty_override(), Some(true));
    }

    #[test]
    fn test_dirty_override_false() {
        let config = OverridesConfig {
            dirty: false,
            no_dirty: true,
            ..Default::default()
        };
        assert_eq!(config.dirty_override(), Some(false));
    }

    #[test]
    fn test_dirty_override_none() {
        let config = OverridesConfig {
            dirty: false,
            no_dirty: false,
            ..Default::default()
        };
        assert_eq!(config.dirty_override(), None);
    }

    #[test]
    fn test_overrides_config_tag_version() {
        let config = OverridesConfig {
            tag_version: Some("v1.2.3".to_string()),
            ..Default::default()
        };
        assert_eq!(config.tag_version, Some("v1.2.3".to_string()));
    }

    #[test]
    fn test_overrides_config_distance() {
        let config = OverridesConfig {
            distance: Some(5),
            ..Default::default()
        };
        assert_eq!(config.distance, Some(5));
    }

    #[test]
    fn test_overrides_config_current_branch() {
        let config = OverridesConfig {
            current_branch: Some("main".to_string()),
            ..Default::default()
        };
        assert_eq!(config.current_branch, Some("main".to_string()));
    }

    #[test]
    fn test_overrides_config_commit_hash() {
        let config = OverridesConfig {
            commit_hash: Some("abc123".to_string()),
            ..Default::default()
        };
        assert_eq!(config.commit_hash, Some("abc123".to_string()));
    }
}
