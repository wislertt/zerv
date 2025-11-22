use clap::Parser;

use crate::cli::common::overrides::CommonOverridesConfig;
use crate::cli::utils::template::Template;

/// Override configuration for flow command
#[derive(Parser, Default, Debug)]
pub struct OverridesConfig {
    #[command(flatten)]
    pub common: CommonOverridesConfig,
}

impl OverridesConfig {
    /// Get post override value or default template
    pub fn override_post(&self) -> Option<Template<u32>> {
        self.common
            .post
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
            assert!(config.common.tag_version.is_none());
            assert!(config.common.distance.is_none());
            assert!(!config.common.dirty);
            assert!(!config.common.no_dirty);
            assert!(!config.common.clean);
            assert!(config.common.bumped_branch.is_none());
            assert!(config.common.bumped_commit_hash.is_none());
            assert!(config.common.bumped_timestamp.is_none());
            assert!(config.common.major.is_none());
            assert!(config.common.minor.is_none());
            assert!(config.common.patch.is_none());
            assert!(config.common.epoch.is_none());
            assert!(config.common.post.is_none());
        }
    }
}
