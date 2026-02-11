use clap::Parser;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
    Validation as CommonValidation,
};

pub mod bumps;
pub mod main;
pub mod overrides;
pub mod resolved;
pub mod validation;

#[cfg(test)]
mod tests {
    pub mod bumps_tests;
    pub mod combination_tests;
    pub mod overrides_tests;
    pub mod resolved_tests;
    pub mod validation_tests;
}

pub use bumps::BumpsConfig;
pub use main::MainConfig;
pub use overrides::OverridesConfig;
pub use resolved::{
    ResolvedArgs,
    ResolvedBumps,
    ResolvedOverrides,
};
use validation::Validation;

/// Generate version from VCS data
#[derive(Parser, Default)]
#[command(about = "Generate version from VCS data")]
#[command(
    long_about = "Generate version strings from version control system data using configurable schemas.
Supports multiple input sources (git, stdin), output formats (semver, pep440, zerv), and VCS overrides
for testing and CI/CD workflows."
)]
#[derive(Debug)]
pub struct VersionArgs {
    #[command(flatten)]
    pub input: InputConfig,

    #[command(flatten)]
    pub output: OutputConfig,

    #[command(flatten)]
    pub main: MainConfig,

    #[command(flatten)]
    pub overrides: OverridesConfig,

    #[command(flatten)]
    pub bumps: BumpsConfig,
}

impl VersionArgs {
    /// Validate arguments and return early errors
    /// This provides early validation before VCS processing
    pub fn validate(&mut self, stdin_content: Option<&str>) -> Result<(), crate::error::ZervError> {
        // Apply smart source default
        self.input
            .apply_smart_source_default(stdin_content.is_some());

        // Use shared validation for input/output
        CommonValidation::validate_io(&self.input, &self.output)?;

        // Validate version-specific modules
        Validation::validate_overrides(&self.overrides)?;
        Validation::validate_bumps(&self.bumps)?;

        // Validate cross-module conflicts
        Validation::validate_cross_module(&self.overrides, &self.bumps)?;

        // Resolve defaults
        Validation::resolve_context_control_defaults(&mut self.bumps)?;
        Validation::resolve_bump_defaults(&mut self.bumps)?;

        Ok(())
    }

    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    pub fn dirty_override(&self) -> Option<bool> {
        self.overrides.dirty_override()
    }
}
