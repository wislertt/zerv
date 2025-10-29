pub mod flow_specific;
pub mod overrides;
pub mod validation;

use clap::Parser;
pub use flow_specific::FlowSpecificConfig;
pub use overrides::OverridesConfig;
pub use validation::Validation;

use crate::cli::common::args::{
    InputConfig,
    OutputConfig,
    Validation as CommonValidation,
};

/// Generate version with intelligent pre-release management based on Git branch patterns
#[derive(Parser)]
#[command(
    about = "Generate version with intelligent pre-release management based on Git branch patterns"
)]
#[command(
    long_about = "Generate version strings with automatic pre-release detection based on Git branch patterns.
This command acts as an intelligent wrapper around 'zerv version' that automatically determines
pre-release information from the current Git branch using configurable pattern matching.

INPUT/OUTPUT OPTIONS:
  -s, --source <TYPE>       Input source: git, stdin
  -f, --input-format <FMT>  Input format: auto, semver, pep440
  -o, --output-format <FMT> Output format: semver, pep440, zerv
  -t, --output-template <TPL> Custom output template (Handlebars)
  -p, --output-prefix <PFX> Add prefix to version output

BRANCH PATTERN DETECTION:
  --branch-rules <RON>      Custom branch pattern rules in RON format
                            Default: develop -> alpha, release/* -> rc

OUTPUT MODES:
  --with-pre-release        Show full version including pre-release (default)
  --base-only               Show only base version without pre-release

POST DISTANCE MODES:
  --post-mode <MODE>        Post calculation: tag (default), commit

OVERRIDES:
  --tag-version <TAG>       Override detected tag version
  --distance <NUM>          Override distance from tag
  --dirty                   Override dirty state to true
  --no-dirty                Override dirty state to false
  --clean                   Force clean state (distance=0, dirty=false)
  --current-branch <NAME>   Override branch name for pattern matching
  --commit-hash <HASH>      Override commit hash

EXAMPLES:
  # Basic flow version with automatic pre-release detection
  zerv flow

  # Custom branch rules (RON format)
  zerv flow --branch-rules \"[(branch_pattern: \"feature/*\", pre_release: \"beta\", number: 2)]\"

  # Different output formats
  zerv flow --output-format pep440
  zerv flow --output-format zerv
  zerv flow --output-prefix v

  # Show base version only
  zerv flow --base-only

  # Override branch for testing
  zerv flow --current-branch release/v1.2.0

  # Force clean release state
  zerv flow --clean

  # Use in different directory
  zerv flow -C /path/to/repo"
)]
#[derive(Debug, Default)]
pub struct FlowArgs {
    #[command(flatten)]
    pub input: InputConfig,

    #[command(flatten)]
    pub output: OutputConfig,

    #[command(flatten)]
    pub flow_specific: FlowSpecificConfig,

    #[command(flatten)]
    pub overrides: OverridesConfig,
}

// TODO: implement tests for this folder.
impl FlowArgs {
    /// Validate arguments and return early errors
    /// This provides early validation before flow processing
    /// Note: source and format validation is handled by clap's value parser
    pub fn validate(&mut self) -> Result<(), crate::error::ZervError> {
        // Use shared validation for input/output
        CommonValidation::validate_io(&self.input, &self.output)?;

        // Validate flow-specific modules
        Validation::validate_flow_specific(&self.flow_specific)?;
        Validation::validate_overrides(&self.overrides)?;
        Validation::validate_branch_rules(&self.flow_specific.branch_rules)?;

        Ok(())
    }

    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    pub fn dirty_override(&self) -> Option<bool> {
        self.overrides.dirty_override()
    }
}
