use clap::Parser;

use crate::constants::{
    SUPPORTED_FORMATS_ARRAY,
    formats,
    pre_release_labels,
    sources,
};
use crate::error::ZervError;

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
    // ============================================================================
    // 1. INPUT CONTROL
    // ============================================================================
    /// Input source for version data
    #[arg(long, default_value = sources::GIT, value_parser = [sources::GIT, sources::STDIN],
          help = "Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)")]
    pub source: String,

    /// Input format for version string parsing
    #[arg(long, default_value = formats::AUTO, value_parser = [formats::AUTO, formats::SEMVER, formats::PEP440],
          help = "Input format: 'auto' (detect), 'semver', or 'pep440'")]
    pub input_format: String,

    /// Change to directory before running command
    #[arg(short = 'C', help = "Change to directory before running command")]
    pub directory: Option<String>,

    // ============================================================================
    // 2. SCHEMA
    // ============================================================================
    /// Schema preset name
    #[arg(long, help = "Schema preset name (standard, calver, etc.)")]
    pub schema: Option<String>,

    /// Custom RON schema definition
    #[arg(long, help = "Custom schema in RON format")]
    pub schema_ron: Option<String>,

    // ============================================================================
    // 3. OVERRIDE
    // ============================================================================
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
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Override dirty state to true (sets dirty=true)")]
    pub dirty: bool,

    /// Override the detected dirty state (sets dirty=false)
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Override dirty state to false (sets dirty=false)")]
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

    // Version component override options
    /// Override major version number
    #[arg(long, help = "Override major version number")]
    pub major: Option<u32>,

    /// Override minor version number
    #[arg(long, help = "Override minor version number")]
    pub minor: Option<u32>,

    /// Override patch version number
    #[arg(long, help = "Override patch version number")]
    pub patch: Option<u32>,

    /// Override epoch number
    #[arg(long, help = "Override epoch number")]
    pub epoch: Option<u32>,

    /// Override post number
    #[arg(long, help = "Override post number")]
    pub post: Option<u32>,

    /// Override dev number
    #[arg(long, help = "Override dev number")]
    pub dev: Option<u32>,

    /// Override pre-release label
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(pre_release_labels::VALID_LABELS),
          help = "Override pre-release label (alpha, beta, rc)")]
    pub pre_release_label: Option<String>,

    /// Override pre-release number
    #[arg(long, help = "Override pre-release number")]
    pub pre_release_num: Option<u32>,

    /// Override custom variables in JSON format
    #[arg(long, help = "Override custom variables in JSON format")]
    pub custom: Option<String>,

    // ============================================================================
    // 4. BUMP
    // ============================================================================
    /// Add to major version (default: 1)
    #[arg(long, help = "Add to major version (default: 1)")]
    pub bump_major: Option<Option<u32>>,

    /// Add to minor version (default: 1)
    #[arg(long, help = "Add to minor version (default: 1)")]
    pub bump_minor: Option<Option<u32>>,

    /// Add to patch version (default: 1)
    #[arg(long, help = "Add to patch version (default: 1)")]
    pub bump_patch: Option<Option<u32>>,

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

    /// Bump pre-release label (alpha, beta, rc) and reset number to 0
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(pre_release_labels::VALID_LABELS),
          help = "Bump pre-release label (alpha, beta, rc) and reset number to 0")]
    pub bump_pre_release_label: Option<String>,

    // Schema-based bump options
    /// Bump core schema component by index and value
    #[arg(
        long,
        value_name = "INDEX VALUE",
        num_args = 2,
        help = "Bump core schema component by index and value (pairs of index, value)"
    )]
    pub bump_core: Vec<u32>,

    /// Bump extra-core schema component by index and value
    #[arg(
        long,
        value_name = "INDEX VALUE",
        num_args = 2,
        help = "Bump extra-core schema component by index and value (pairs of index, value)"
    )]
    pub bump_extra_core: Vec<u32>,

    /// Bump build schema component by index and value
    #[arg(
        long,
        value_name = "INDEX VALUE",
        num_args = 2,
        help = "Bump build schema component by index and value (pairs of index, value)"
    )]
    pub bump_build: Vec<u32>,

    // Context control options
    /// Include VCS context qualifiers (default behavior)
    #[arg(long, help = "Include VCS context qualifiers (default behavior)")]
    pub bump_context: bool,

    /// Pure tag version, no VCS context
    #[arg(long, help = "Pure tag version, no VCS context")]
    pub no_bump_context: bool,

    // ============================================================================
    // 5. OUTPUT CONTROL
    // ============================================================================
    /// Output format for generated version
    #[arg(long, default_value = formats::SEMVER, value_parser = SUPPORTED_FORMATS_ARRAY,
          help = format!("Output format: '{}' (default), '{}', or '{}' (RON format for piping)", formats::SEMVER, formats::PEP440, formats::ZERV))]
    pub output_format: String,

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
}

impl Default for VersionArgs {
    fn default() -> Self {
        Self {
            source: sources::GIT.to_string(),
            major: None,
            minor: None,
            patch: None,
            schema: None,
            schema_ron: None,
            input_format: formats::AUTO.to_string(),
            output_format: formats::SEMVER.to_string(),
            tag_version: None,
            distance: None,
            dirty: false,
            no_dirty: false,
            clean: false,
            current_branch: None,
            commit_hash: None,
            post: None,
            dev: None,
            pre_release_label: None,
            pre_release_num: None,
            epoch: None,
            custom: None,
            bump_major: None,
            bump_minor: None,
            bump_patch: None,
            bump_post: None,
            bump_dev: None,
            bump_pre_release_num: None,
            bump_epoch: None,
            bump_pre_release_label: None,
            bump_core: Vec::new(),
            bump_extra_core: Vec::new(),
            bump_build: Vec::new(),
            bump_context: false,
            no_bump_context: false,
            output_template: None,
            output_prefix: None,
            directory: None,
        }
    }
}

impl VersionArgs {
    /// Validate arguments and return early errors
    /// This provides early validation before VCS processing
    pub fn validate(&mut self) -> Result<(), ZervError> {
        // Check for conflicting dirty flags
        if self.dirty && self.no_dirty {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --dirty with --no-dirty (conflicting options)".to_string(),
            ));
        }

        // Check for conflicting context control and dirty flags
        if self.no_bump_context && self.dirty {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --no-bump-context with --dirty (conflicting options)".to_string(),
            ));
        }

        // Check for --clean conflicts
        if self.clean {
            if self.distance.is_some() {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --distance (conflicting options)".to_string(),
                ));
            }
            if self.dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --dirty (conflicting options)".to_string(),
                ));
            }
            if self.no_dirty {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --clean with --no-dirty (conflicting options)".to_string(),
                ));
            }
        }

        // Resolve default context control behavior
        self.resolve_context_control_defaults()?;

        // Resolve default bump values
        self.resolve_bump_defaults()?;

        // Validate pre-release flags
        self.validate_pre_release_flags()?;

        // Validate schema-based bump arguments
        self.validate_schema_bump_args()?;

        Ok(())
    }

    /// Resolve default context control behavior
    /// If neither --bump-context nor --no-bump-context is provided, default to --bump-context
    fn resolve_context_control_defaults(&mut self) -> Result<(), ZervError> {
        // Mathematical approach: handle all possible states
        match (self.bump_context, self.no_bump_context) {
            // Invalid case: both flags provided
            (true, true) => {
                return Err(ZervError::ConflictingOptions(
                    "Cannot use --bump-context with --no-bump-context (conflicting options)"
                        .to_string(),
                ));
            }
            // Default case: neither flag provided
            (false, false) => {
                self.bump_context = true;
            }
            // Any other case: explicit flags provided (keep as is)
            _ => {
                // No change needed - already correct
            }
        }

        Ok(())
    }

    /// Resolve default bump values
    /// If a bump option is provided without a value, set it to 1 (the default)
    fn resolve_bump_defaults(&mut self) -> Result<(), ZervError> {
        // Resolve bump_major: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_major {
            self.bump_major = Some(Some(1));
        }

        // Resolve bump_minor: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_minor {
            self.bump_minor = Some(Some(1));
        }

        // Resolve bump_patch: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_patch {
            self.bump_patch = Some(Some(1));
        }

        // Resolve bump_post: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_post {
            self.bump_post = Some(Some(1));
        }

        // Resolve bump_dev: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_dev {
            self.bump_dev = Some(Some(1));
        }

        // Resolve bump_pre_release_num: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_pre_release_num {
            self.bump_pre_release_num = Some(Some(1));
        }

        // Resolve bump_epoch: Some(None) -> Some(Some(1))
        if let Some(None) = self.bump_epoch {
            self.bump_epoch = Some(Some(1));
        }

        Ok(())
    }

    /// Validate pre-release flags for conflicts
    fn validate_pre_release_flags(&self) -> Result<(), ZervError> {
        if self.pre_release_label.is_some() && self.bump_pre_release_label.is_some() {
            return Err(ZervError::ConflictingOptions(
                "Cannot use --pre-release-label with --bump-pre-release-label".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate schema-based bump arguments
    fn validate_schema_bump_args(&self) -> Result<(), ZervError> {
        // Validate bump_core arguments (must be pairs of index, value)
        if !self.bump_core.len().is_multiple_of(2) {
            return Err(ZervError::InvalidArgument(
                "--bump-core requires pairs of index and value arguments".to_string(),
            ));
        }

        // Validate bump_extra_core arguments (must be pairs of index, value)
        if !self.bump_extra_core.len().is_multiple_of(2) {
            return Err(ZervError::InvalidArgument(
                "--bump-extra-core requires pairs of index and value arguments".to_string(),
            ));
        }

        // Validate bump_build arguments (must be pairs of index, value)
        if !self.bump_build.len().is_multiple_of(2) {
            return Err(ZervError::InvalidArgument(
                "--bump-build requires pairs of index and value arguments".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the dirty override state (None = use VCS, Some(bool) = override)
    pub fn dirty_override(&self) -> Option<bool> {
        match (self.dirty, self.no_dirty) {
            (true, false) => Some(true),    // --dirty
            (false, true) => Some(false),   // --no-dirty
            (false, false) => None,         // neither (use VCS)
            (true, true) => unreachable!(), // Should be caught by validation
        }
    }

    /// Resolve schema selection with default fallback
    /// Returns (schema_name, schema_ron) with default applied if neither is provided
    pub fn resolve_schema(&self) -> (Option<&str>, Option<&str>) {
        match (self.schema.as_deref(), self.schema_ron.as_deref()) {
            (Some(name), None) => (Some(name), None),
            (None, Some(ron)) => (None, Some(ron)),
            (Some(_), Some(_)) => (self.schema.as_deref(), self.schema_ron.as_deref()), // Both provided - let validation handle conflict
            (None, None) => (Some("zerv-standard"), None), // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::constants::{
        formats,
        sources,
    };
    use crate::test_utils::VersionArgsFixture;

    #[test]
    fn test_version_args_defaults() {
        let args = VersionArgs::try_parse_from(["version"]).unwrap();
        assert_eq!(args.source, sources::GIT);
        assert!(args.schema.is_none());
        assert!(args.schema_ron.is_none());
        assert_eq!(args.input_format, formats::AUTO);
        assert_eq!(args.output_format, formats::SEMVER);

        // VCS override options should be None/false by default
        assert!(args.tag_version.is_none());
        assert!(args.distance.is_none());
        assert!(!args.dirty);
        assert!(!args.no_dirty);
        assert!(!args.clean);
        assert!(args.current_branch.is_none());
        assert!(args.commit_hash.is_none());
        assert!(args.post.is_none());
        assert!(args.dev.is_none());
        assert!(args.pre_release_label.is_none());
        assert!(args.pre_release_num.is_none());
        assert!(args.epoch.is_none());
        assert!(args.custom.is_none());

        // Bump options should be None by default
        assert!(args.bump_major.is_none());
        assert!(args.bump_minor.is_none());
        assert!(args.bump_patch.is_none());
        assert!(args.bump_post.is_none());
        assert!(args.bump_dev.is_none());
        assert!(args.bump_pre_release_num.is_none());
        assert!(args.bump_epoch.is_none());
        assert!(args.bump_pre_release_label.is_none());

        // Schema-based bump options should be empty by default
        assert!(args.bump_core.is_empty());
        assert!(args.bump_extra_core.is_empty());
        assert!(args.bump_build.is_empty());

        // Context control options should be false by default
        assert!(!args.bump_context);
        assert!(!args.no_bump_context);

        // Output options should be None by default
        assert!(args.output_template.is_none());
        assert!(args.output_prefix.is_none());
    }

    #[test]
    fn test_version_args_with_overrides() {
        let args = VersionArgs::try_parse_from([
            "zerv",
            "--tag-version",
            "v2.0.0",
            "--distance",
            "5",
            "--dirty",
            "--current-branch",
            "feature/test",
            "--commit-hash",
            "abc123",
            "--input-format",
            "semver",
            "--output-prefix",
            "version:",
        ])
        .unwrap();

        assert_eq!(args.tag_version, Some("v2.0.0".to_string()));
        assert_eq!(args.distance, Some(5));
        assert!(args.dirty);
        assert!(!args.no_dirty);
        assert!(!args.clean);
        assert_eq!(args.current_branch, Some("feature/test".to_string()));
        assert_eq!(args.commit_hash, Some("abc123".to_string()));
        assert_eq!(args.input_format, formats::SEMVER);
        assert_eq!(args.output_prefix, Some("version:".to_string()));
    }

    #[test]
    fn test_version_args_clean_flag() {
        let args = VersionArgs::try_parse_from(["version", "--clean"]).unwrap();

        assert!(args.clean);
        assert!(args.distance.is_none());
        assert!(!args.dirty);
        assert!(!args.no_dirty);
    }

    #[test]
    fn test_version_args_dirty_flags() {
        // Test --dirty flag
        let args = VersionArgs::try_parse_from(["version", "--dirty"]).unwrap();
        assert!(args.dirty);
        assert!(!args.no_dirty);

        // Test --no-dirty flag
        let args = VersionArgs::try_parse_from(["version", "--no-dirty"]).unwrap();
        assert!(!args.dirty);
        assert!(args.no_dirty);

        // Test both flags together should fail early validation
        let mut args = VersionArgs::try_parse_from(["version", "--dirty", "--no-dirty"]).unwrap();
        assert!(args.dirty);
        assert!(args.no_dirty);

        // The conflict should be caught by early validation
        let result = args.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_dirty_override_helper() {
        // Test --dirty flag
        let args = VersionArgs::try_parse_from(["version", "--dirty"]).unwrap();
        assert_eq!(args.dirty_override(), Some(true));

        // Test --no-dirty flag
        let args = VersionArgs::try_parse_from(["version", "--no-dirty"]).unwrap();
        assert_eq!(args.dirty_override(), Some(false));

        // Test neither flag (use VCS)
        let args = VersionArgs::try_parse_from(["version"]).unwrap();
        assert_eq!(args.dirty_override(), None);
    }

    #[test]
    fn test_validate_no_conflicts() {
        // Test with no conflicting options
        let mut args = VersionArgs::try_parse_from(["version"]).unwrap();
        assert!(args.validate().is_ok());

        // Test with individual options (no conflicts)
        let mut args = VersionArgs::try_parse_from(["version", "--dirty"]).unwrap();
        assert!(args.validate().is_ok());

        let mut args = VersionArgs::try_parse_from(["version", "--no-dirty"]).unwrap();
        assert!(args.validate().is_ok());

        let mut args = VersionArgs::try_parse_from(["version", "--clean"]).unwrap();
        assert!(args.validate().is_ok());

        let mut args = VersionArgs::try_parse_from(["version", "--distance", "5"]).unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_dirty_conflicts() {
        // Test conflicting dirty flags
        let mut args = VersionArgs::try_parse_from(["version", "--dirty", "--no-dirty"]).unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--dirty"));
        assert!(error.to_string().contains("--no-dirty"));
        assert!(error.to_string().contains("conflicting options"));
    }

    #[test]
    fn test_validate_clean_conflicts() {
        // Test --clean with --distance
        let mut args =
            VersionArgs::try_parse_from(["version", "--clean", "--distance", "5"]).unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--clean"));
        assert!(error.to_string().contains("--distance"));

        // Test --clean with --dirty
        let mut args = VersionArgs::try_parse_from(["version", "--clean", "--dirty"]).unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--clean"));
        assert!(error.to_string().contains("--dirty"));

        // Test --clean with --no-dirty
        let mut args = VersionArgs::try_parse_from(["version", "--clean", "--no-dirty"]).unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--clean"));
        assert!(error.to_string().contains("--no-dirty"));
    }

    #[test]
    fn test_validate_context_control_conflicts() {
        // Test conflicting context control flags
        let mut args =
            VersionArgs::try_parse_from(["version", "--bump-context", "--no-bump-context"])
                .unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--bump-context"));
        assert!(error.to_string().contains("--no-bump-context"));
        assert!(error.to_string().contains("conflicting options"));
    }

    #[test]
    fn test_validate_clean_with_non_conflicting_options() {
        // Test --clean with options that should NOT conflict
        let mut args = VersionArgs::try_parse_from([
            "zerv",
            "--clean",
            "--tag-version",
            "v2.0.0",
            "--current-branch",
            "main",
            "--commit-hash",
            "abc123",
        ])
        .unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_no_bump_context_with_dirty_conflict() {
        // Test --no-bump-context with --dirty (should conflict)
        let mut args =
            VersionArgs::try_parse_from(["zerv", "--no-bump-context", "--dirty"]).unwrap();
        let result = args.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--no-bump-context"));
        assert!(error.to_string().contains("--dirty"));
        assert!(error.to_string().contains("conflicting options"));
    }

    #[test]
    fn test_validate_schema_bump_args_valid() {
        // Test valid schema bump arguments (pairs of index, value)
        let args = VersionArgs::try_parse_from([
            "version",
            "--bump-core",
            "0",
            "1",
            "--bump-core",
            "2",
            "3",
            "--bump-extra-core",
            "1",
            "5",
            "--bump-build",
            "0",
            "10",
            "--bump-build",
            "1",
            "20",
        ])
        .unwrap();

        let mut args = args;
        assert!(args.validate().is_ok());
        assert_eq!(args.bump_core, vec![0, 1, 2, 3]);
        assert_eq!(args.bump_extra_core, vec![1, 5]);
        assert_eq!(args.bump_build, vec![0, 10, 1, 20]);
    }

    #[test]
    fn test_validate_schema_bump_args_invalid_odd_count() {
        // Test invalid schema bump arguments (odd number of arguments)
        // We need to manually create the args with odd count since clap validates pairs
        let mut args = VersionArgs {
            bump_core: vec![0, 1, 2], // Odd count: 3 elements
            ..Default::default()
        };
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::InvalidArgument(_)));
        assert!(error.to_string().contains("--bump-core requires pairs"));
    }

    #[test]
    fn test_validate_schema_bump_args_empty() {
        // Test empty schema bump arguments (should be valid)
        let args = VersionArgs::try_parse_from(["version"]).unwrap();
        let mut args = args;
        assert!(args.validate().is_ok());
        assert!(args.bump_core.is_empty());
        assert!(args.bump_extra_core.is_empty());
        assert!(args.bump_build.is_empty());
    }

    #[test]
    fn test_validate_multiple_conflicts() {
        // Test that validation fails on the first conflict found
        let mut args = VersionArgs::try_parse_from([
            "zerv",
            "--clean",
            "--distance",
            "5",
            "--dirty",
            "--no-dirty",
        ])
        .unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        // Should fail on the first conflict (dirty flags conflict comes first)
        assert!(error_msg.contains("--dirty"));
        assert!(error_msg.contains("--no-dirty"));
        assert!(error_msg.contains("conflicting options"));
    }

    #[test]
    fn test_validate_error_message_quality() {
        // Test that error messages are clear and actionable
        let mut args = VersionArgs::try_parse_from(["version", "--dirty", "--no-dirty"]).unwrap();
        let result = args.validate();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Conflicting options"));
        assert!(error_msg.contains("--dirty"));
        assert!(error_msg.contains("--no-dirty"));
        assert!(error_msg.contains("conflicting options"));
        assert!(error_msg.contains("Cannot use"));
    }

    #[test]
    fn test_context_control_all_scenarios() {
        // Test all 4 possible states of (bump_context, no_bump_context)

        // Scenario 1: (false, false) - Neither flag provided: should default to bump-context
        let mut args = VersionArgs::try_parse_from(["version"]).unwrap();
        assert!(!args.bump_context);
        assert!(!args.no_bump_context);
        assert!(args.validate().is_ok());
        assert!(args.bump_context);
        assert!(!args.no_bump_context);

        // Scenario 2: (true, false) - Explicit --bump-context: should remain unchanged
        let mut args = VersionArgs::try_parse_from(["version", "--bump-context"]).unwrap();
        assert!(args.bump_context);
        assert!(!args.no_bump_context);
        assert!(args.validate().is_ok());
        assert!(args.bump_context);
        assert!(!args.no_bump_context);

        // Scenario 3: (false, true) - Explicit --no-bump-context: should remain unchanged
        let mut args = VersionArgs::try_parse_from(["version", "--no-bump-context"]).unwrap();
        assert!(!args.bump_context);
        assert!(args.no_bump_context);
        assert!(args.validate().is_ok());
        assert!(!args.bump_context);
        assert!(args.no_bump_context);

        // Scenario 4: (true, true) - Both flags provided: should return error
        let mut args =
            VersionArgs::try_parse_from(["version", "--bump-context", "--no-bump-context"])
                .unwrap();
        assert!(args.bump_context);
        assert!(args.no_bump_context);
        let result = args.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--bump-context"));
        assert!(error.to_string().contains("--no-bump-context"));
    }

    #[test]
    fn test_version_args_fixture() {
        let args = VersionArgsFixture::new().build();
        assert_eq!(args.source, sources::GIT);
        assert_eq!(args.output_format, formats::SEMVER);

        let args_with_overrides = VersionArgsFixture::new()
            .with_tag_version("v2.0.0")
            .with_distance(5)
            .with_dirty(true)
            .build();
        assert_eq!(args_with_overrides.tag_version, Some("v2.0.0".to_string()));
        assert_eq!(args_with_overrides.distance, Some(5));
        assert!(args_with_overrides.dirty);

        let args_with_clean = VersionArgsFixture::new().with_clean_flag(true).build();
        assert!(args_with_clean.clean);

        let args_with_bumps = VersionArgsFixture::new()
            .with_bump_major(1)
            .with_bump_minor(1)
            .with_bump_patch(1)
            .build();
        assert!(args_with_bumps.bump_major.is_some());
        assert!(args_with_bumps.bump_minor.is_some());
        assert!(args_with_bumps.bump_patch.is_some());
    }

    #[test]
    fn test_validate_pre_release_flag_conflicts() {
        // Test conflicting pre-release flags
        let mut args = VersionArgsFixture::new()
            .with_pre_release_label("alpha")
            .with_bump_pre_release_label("beta")
            .build();
        let result = args.validate();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ZervError::ConflictingOptions(_)));
        assert!(error.to_string().contains("--pre-release-label"));
        assert!(error.to_string().contains("--bump-pre-release-label"));
        assert!(error.to_string().contains("Cannot use"));
    }

    #[test]
    fn test_validate_pre_release_flags_no_conflict() {
        // Test that individual pre-release flags don't conflict
        let mut args = VersionArgsFixture::new()
            .with_pre_release_label("alpha")
            .build();
        assert_eq!(args.pre_release_label, Some("alpha".to_string()));
        assert_eq!(args.bump_pre_release_label, None);
        assert!(args.validate().is_ok());

        let mut args = VersionArgsFixture::new()
            .with_bump_pre_release_label("beta")
            .build();
        assert_eq!(args.pre_release_label, None);
        assert_eq!(args.bump_pre_release_label, Some("beta".to_string()));
        assert!(args.validate().is_ok());
    }
}
