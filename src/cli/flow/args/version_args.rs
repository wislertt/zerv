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
use crate::utils::constants::post_modes;
use crate::version::zerv::core::Zerv;

impl FlowArgs {
    /// Create base VersionArgs with shared configuration
    fn create_version_args(&self, bumps: BumpsConfig, override_dirty: bool) -> VersionArgs {
        VersionArgs {
            input: self.input.clone(),
            output: OutputConfig::zerv(),
            main: MainConfig::from_schema(self.schema.clone()),
            overrides: OverridesConfig {
                common: {
                    let mut common_config = self.overrides.common.clone();
                    common_config.post = self.overrides.override_post();
                    common_config.dirty = override_dirty;
                    common_config
                },
                ..Default::default()
            },
            bumps,
        }
    }

    pub fn override_dirty(
        &self,
        current_dirty: Option<bool>,
        current_distance: Option<u64>,
    ) -> bool {
        if !self.overrides.common.dirty && !self.overrides.common.no_dirty {
            if self.post_mode() == post_modes::TAG
                && (current_dirty == Some(true) || current_distance.unwrap_or(0) > 0)
            {
                true
            } else {
                self.overrides.common.dirty
            }
        } else {
            self.overrides.common.dirty
        }
    }

    /// Get current zerv object "as-is" (no bumps)
    pub fn get_current_zerv_object(&self, stdin_content: Option<&str>) -> Result<Zerv, ZervError> {
        let version_args =
            self.create_version_args(BumpsConfig::default(), self.overrides.common.dirty);

        let ron_output = run_version_pipeline(version_args, stdin_content)?;
        from_str(&ron_output)
            .map_err(|e| ZervError::InvalidFormat(format!("Failed to parse version output: {}", e)))
    }

    /// Create bumped version args for final pipeline
    pub fn create_bumped_version_args(
        &self,
        current_zerv: &Zerv,
    ) -> Result<VersionArgs, ZervError> {
        let bumps = BumpsConfig {
            bump_pre_release_label: self.bump_pre_release_label(),
            bump_pre_release_num: self.bump_pre_release_num(),
            bump_patch: self.bump_patch(),
            bump_post: self.bump_post(),
            bump_dev: self.bump_dev(),
            ..Default::default()
        };

        Ok(self.create_version_args(
            bumps,
            self.override_dirty(current_zerv.vars.dirty, current_zerv.vars.distance),
        ))
    }
}
