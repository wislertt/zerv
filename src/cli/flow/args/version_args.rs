use ron::from_str;

use super::FlowArgs;
use crate::cli::common::args::OutputConfig;
use crate::cli::version::args::{
    BumpsConfig,
    MainConfig,
    VersionArgs,
};
use crate::cli::version::pipeline::run_version_pipeline;
use crate::error::ZervError;
use crate::version::zerv::core::Zerv;

impl FlowArgs {
    /// Get current zerv object "as-is" (no bumps)
    pub fn get_current_zerv_object(&self) -> Result<Zerv, ZervError> {
        let version_args = VersionArgs {
            input: self.input.clone(),
            output: OutputConfig::zerv(),
            main: MainConfig::from_schema(self.schema.clone()),
            overrides: Default::default(),
            bumps: BumpsConfig::default(),
        };

        let ron_output = run_version_pipeline(version_args, None)?;
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
            overrides: Default::default(),
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
