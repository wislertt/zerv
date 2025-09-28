use crate::constants::{SUPPORTED_FORMATS_ARRAY, formats, sources};
use clap::Parser;

#[derive(Parser)]
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
pub struct VersionArgs {
    /// Version string (deprecated - use --tag-version instead)
    #[arg(help = "Version string (deprecated - use --tag-version instead)")]
    pub version: Option<String>,

    /// Input source for version data
    #[arg(long, default_value = sources::GIT, value_parser = [sources::GIT, sources::STDIN],
          help = "Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)")]
    pub source: String,

    /// Schema preset name
    #[arg(long, help = "Schema preset name (standard, calver, etc.)")]
    pub schema: Option<String>,

    /// Custom RON schema definition
    #[arg(long, help = "Custom schema in RON format")]
    pub schema_ron: Option<String>,

    /// Input format for version string parsing
    #[arg(long, default_value = formats::AUTO, value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440, formats::ZERV],
          help = "Input format: 'auto' (detect), 'semver', 'pep440', or 'zerv' (for stdin)")]
    pub input_format: String,

    /// Output format for generated version
    #[arg(long, default_value = formats::SEMVER, value_parser = SUPPORTED_FORMATS_ARRAY,
          help = format!("Output format: '{}' (default), '{}', or '{}' (RON format for piping)", formats::SEMVER, formats::PEP440, formats::ZERV))]
    pub output_format: String,

    // VCS override options
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
    #[arg(long, help = "Override dirty state to true (sets dirty=true)")]
    pub dirty: bool,

    /// Override the detected dirty state (sets dirty=false)
    #[arg(long, help = "Override dirty state to false (sets dirty=false)")]
    pub no_dirty: bool,

    /// Set distance=0 and dirty=false (clean release state)
    #[arg(
        long,
        help = "Force clean release state (sets distance=0, dirty=false). Conflicts with --distance and --dirty"
    )]
    pub clean: bool,

    /// Override the detected current branch name
    #[arg(long, help = "Override current branch name")]
    pub current_branch: Option<String>,

    /// Override the detected commit hash
    #[arg(long, help = "Override commit hash (full or short form)")]
    pub commit_hash: Option<String>,

    // Additional override options
    /// Override post number
    #[arg(long, help = "Override post number")]
    pub post: Option<u32>,

    /// Override dev number
    #[arg(long, help = "Override dev number")]
    pub dev: Option<u32>,

    /// Override pre-release label
    #[arg(long, help = "Override pre-release label (alpha, beta, rc, etc.)")]
    pub pre_release_label: Option<String>,

    /// Override pre-release number
    #[arg(long, help = "Override pre-release number")]
    pub pre_release_num: Option<u32>,

    /// Override epoch number
    #[arg(long, help = "Override epoch number")]
    pub epoch: Option<u32>,

    /// Override custom variables in JSON format
    #[arg(long, help = "Override custom variables in JSON format")]
    pub custom: Option<String>,

    // Bump options (relative modifications)
    /// Add to major version (default: 1)
    #[arg(long, help = "Add to major version (default: 1)")]
    pub bump_major: Option<Option<u32>>,

    /// Add to minor version (default: 1)
    #[arg(long, help = "Add to minor version (default: 1)")]
    pub bump_minor: Option<Option<u32>>,

    /// Add to patch version (default: 1)
    #[arg(long, help = "Add to patch version (default: 1)")]
    pub bump_patch: Option<Option<u32>>,

    /// Add to distance from tag (default: 1)
    #[arg(long, help = "Add to distance from tag (default: 1)")]
    pub bump_distance: Option<Option<u32>>,

    /// Add to post number (default: 1)
    #[arg(long, help = "Add to post number (default: 1)")]
    pub bump_post: Option<Option<u32>>,

    /// Add to dev number (default: 1)
    #[arg(long, help = "Add to dev number (default: 1)")]
    pub bump_dev: Option<Option<u32>>,

    /// Add to pre-release number (default: 1)
    #[arg(long, help = "Add to pre-release number (default: 1)")]
    pub bump_pre_release_num: Option<Option<u32>>,

    /// Add to epoch number (default: 1)
    #[arg(long, help = "Add to epoch number (default: 1)")]
    pub bump_epoch: Option<Option<u32>>,

    // Context control options
    /// Include VCS context qualifiers (default behavior)
    #[arg(long, help = "Include VCS context qualifiers (default behavior)")]
    pub bump_context: bool,

    /// Pure tag version, no VCS context
    #[arg(long, help = "Pure tag version, no VCS context")]
    pub no_bump_context: bool,

    // Output options for future extension
    /// Output template for custom formatting (future extension)
    #[arg(
        long,
        help = "Output template for custom formatting (future extension)"
    )]
    pub output_template: Option<String>,

    /// Prefix to add to output
    #[arg(
        long,
        help = "Prefix to add to version output (e.g., 'v' for 'v1.0.0')"
    )]
    pub output_prefix: Option<String>,

    /// Change to directory before running command
    #[arg(short = 'C', help = "Change to directory before running command")]
    pub directory: Option<String>,
}

impl VersionArgs {
    /// Check if any VCS overrides are specified in the arguments
    pub fn has_overrides(&self) -> bool {
        self.tag_version.is_some()
            || self.distance.is_some()
            || self.dirty
            || self.no_dirty
            || self.clean
            || self.current_branch.is_some()
            || self.commit_hash.is_some()
            || self.post.is_some()
            || self.dev.is_some()
            || self.pre_release_label.is_some()
            || self.pre_release_num.is_some()
            || self.epoch.is_some()
            || self.custom.is_some()
    }

    /// Check if any bump operations are specified in the arguments
    pub fn has_bumps(&self) -> bool {
        self.bump_major.is_some()
            || self.bump_minor.is_some()
            || self.bump_patch.is_some()
            || self.bump_distance.is_some()
            || self.bump_post.is_some()
            || self.bump_dev.is_some()
            || self.bump_pre_release_num.is_some()
            || self.bump_epoch.is_some()
    }

    /// Check if any context control options are specified
    pub fn has_context_control(&self) -> bool {
        self.bump_context || self.no_bump_context
    }
}
