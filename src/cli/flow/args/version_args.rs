use ron::from_str;

use super::FlowArgs;
use crate::cli::common::args::OutputConfig;
use crate::cli::version::args::{
    BumpsConfig,
    MainConfig,
    OverridesConfig,
    VersionArgs,
};
use crate::cli::version::pipeline::run_version_pipeline;
use crate::error::ZervError;
use crate::version::zerv::core::Zerv;

impl FlowArgs {
    /// Get current zerv object "as-is" (no bumps)
    pub fn get_current_zerv_object(&self, stdin_content: Option<&str>) -> Result<Zerv, ZervError> {
        let version_args = VersionArgs {
            input: self.input.clone(),
            output: OutputConfig::zerv(),
            main: MainConfig::from_schema(self.schema.clone()),
            overrides: OverridesConfig {
                bumped_branch: self.overrides.bumped_branch.clone(),
                ..Default::default()
            },
            bumps: BumpsConfig::default(),
        };

        let ron_output = run_version_pipeline(version_args, stdin_content)?;
        from_str(&ron_output)
            .map_err(|e| ZervError::InvalidFormat(format!("Failed to parse version output: {}", e)))
    }

    /// Create bumped version args for final pipeline
    pub fn create_bumped_version_args(
        &self,
        _current_zerv: &Zerv,
    ) -> Result<VersionArgs, ZervError> {
        Ok(VersionArgs {
            input: self.input.clone(),
            output: OutputConfig::zerv(),
            main: MainConfig::from_schema(self.schema.clone()),
            overrides: OverridesConfig {
                tag_version: self.overrides.tag_version.clone(),
                distance: self.overrides.distance,
                dirty: self.overrides.dirty,
                no_dirty: self.overrides.no_dirty,
                clean: self.overrides.clean,
                bumped_branch: self.overrides.bumped_branch.clone(),
                bumped_commit_hash: self.overrides.bumped_commit_hash.clone(),
                bumped_timestamp: self.overrides.bumped_timestamp,
                major: self.overrides.major.clone(),
                minor: self.overrides.minor.clone(),
                patch: self.overrides.patch.clone(),
                epoch: self.overrides.epoch.clone(),
                post: self.overrides.post.clone(),
                dev: self.overrides.dev.clone(),
                pre_release_label: None, // Flow handles pre-release through branch rules
                pre_release_num: None,   // Flow handles pre-release through branch rules
                custom: None,            // Not supported in flow
                core: vec![],            // Not supported in flow
                extra_core: vec![],      // Not supported in flow
                build: vec![],           // Not supported in flow
            },
            bumps: BumpsConfig {
                bump_pre_release_label: self.bump_pre_release_label(),
                bump_pre_release_num: self.bump_pre_release_num(),
                bump_patch: self.bump_patch(),
                bump_post: self.bump_post(),
                bump_dev: self.bump_dev(),
                ..Default::default()
            },
        })
    }
}
