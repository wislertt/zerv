use clap::Parser;

pub mod bumps;
pub mod main;
pub mod overrides;
pub mod resolved;
pub mod validation;

#[cfg(test)]
mod tests {
    pub mod bumps_tests;
    pub mod combination_tests;
    pub mod main_tests;
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

INPUT SOURCES:
  --source git     Extract version data from git repository (default)
  --source stdin   Read Zerv RON format from stdin for piping workflows

OUTPUT FORMATS:
  --output-format semver   Semantic Versioning format (default)
  --output-format pep440   Python PEP440 format
  --output-format zerv     Zerv RON format for piping

VCS OVERRIDES:
  Override detected VCS values for testing and simulation:
  --tag-version <TAG>      Override detected tag version
  --distance <NUM>         Override distance from tag
  --dirty                  Override dirty state to true
  --no-dirty               Override dirty state to false
  --clean                  Force clean state (distance=0, dirty=false)
  --current-branch <NAME>  Override branch name
  --commit-hash <HASH>     Override commit hash

EXAMPLES:
  # Basic version generation
  zerv version

  # Generate PEP440 format with calver schema
  zerv version --output-format pep440 --schema calver

  # Override VCS values for testing
  zerv version --tag-version v2.0.0 --distance 5 --dirty
  zerv version --tag-version v2.0.0 --distance 5 --no-dirty

  # Force clean release state
  zerv version --clean

  # Use in different directory
  zerv version -C /path/to/repo

  # Pipe between commands with full data preservation
  zerv version --output-format zerv | zerv version --source stdin --schema calver

  # Parse specific input format
  zerv version --tag-version 2.0.0-alpha.1 --input-format semver"
)]
#[derive(Debug)]
pub struct VersionArgs {
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
    pub fn validate(&mut self) -> Result<(), crate::error::ZervError> {
        // Validate individual modules
        Validation::validate_main(&self.main)?;
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
