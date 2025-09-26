use crate::cli::utils::format_handler::InputFormatHandler;
use crate::cli::utils::fuzzy_bool::FuzzyBool;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::cli::utils::vcs_override::VcsOverrideProcessor;
use crate::constants::{FORMAT_PEP440, FORMAT_SEMVER, FORMAT_ZERV, SUPPORTED_FORMATS_ARRAY};
use crate::error::ZervError;
use crate::pipeline::vcs_data_to_zerv_vars;
use crate::schema::create_zerv_version;
use crate::version::Zerv;
use clap::Parser;
use std::env::current_dir;

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
  --dirty <BOOL>           Override dirty state (true/false, yes/no, 1/0, etc.)
  --clean                  Force clean state (distance=0, dirty=false)
  --current-branch <NAME>  Override branch name
  --commit-hash <HASH>     Override commit hash

EXAMPLES:
  # Basic version generation
  zerv version

  # Generate PEP440 format with calver schema
  zerv version --output-format pep440 --schema calver

  # Override VCS values for testing
  zerv version --tag-version v2.0.0 --distance 5 --dirty yes

  # Force clean release state
  zerv version --clean

  # Use in different directory
  zerv version -C /path/to/repo

  # Pipe between commands with full data preservation
  zerv version --output-format zerv | zerv version --source stdin --input-format zerv --schema calver

  # Parse specific input format
  zerv version --tag-version 2.0.0-alpha.1 --input-format semver"
)]
pub struct VersionArgs {
    /// Version string (deprecated - use --tag-version instead)
    #[arg(help = "Version string (deprecated - use --tag-version instead)")]
    pub version: Option<String>,

    /// Input source for version data
    #[arg(long, default_value = "git", value_parser = ["git", "stdin"],
          help = "Input source: 'git' (extract from repository) or 'stdin' (read Zerv RON format)")]
    pub source: String,

    /// Schema preset name
    #[arg(long, help = "Schema preset name (standard, calver, etc.)")]
    pub schema: Option<String>,

    /// Custom RON schema definition
    #[arg(long, help = "Custom schema in RON format")]
    pub schema_ron: Option<String>,

    /// Input format for version string parsing
    #[arg(long, default_value = "auto", value_parser = ["auto", "semver", "pep440", "zerv"],
          help = "Input format: 'auto' (detect), 'semver', 'pep440', or 'zerv' (for stdin RON input)")]
    pub input_format: String,

    /// Output format for generated version
    #[arg(long, default_value = FORMAT_SEMVER, value_parser = SUPPORTED_FORMATS_ARRAY,
          help = format!("Output format: '{}' (default), '{}', or '{}' (RON format for piping)", FORMAT_SEMVER, FORMAT_PEP440, FORMAT_ZERV))]
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

    /// Override the detected dirty state
    #[arg(
        long,
        help = "Override dirty state. Accepts: true/false, yes/no, y/n, 1/0, on/off (case-insensitive)"
    )]
    pub dirty: Option<FuzzyBool>,

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
            || self.dirty.is_some()
            || self.clean
            || self.current_branch.is_some()
            || self.commit_hash.is_some()
    }
}

pub fn run_version_pipeline(args: VersionArgs) -> Result<String, ZervError> {
    // 1. Determine working directory
    let work_dir = match args.directory.as_deref() {
        Some(dir) => std::path::PathBuf::from(dir),
        None => current_dir()?,
    };

    // 2. Resolve input source and get Zerv object
    let mut zerv_object = match args.source.as_str() {
        "git" => {
            // Get git VCS data
            // If directory was specified via -C, only look in that directory (depth 0)
            // If no directory specified, allow unlimited depth search
            let max_depth = if args.directory.is_some() {
                Some(0)
            } else {
                None
            };
            let mut vcs_data =
                crate::vcs::detect_vcs_with_limit(&work_dir, max_depth)?.get_vcs_data()?;

            // Parse git tag with input format if available and validate it
            if let Some(ref tag_version) = vcs_data.tag_version {
                let _parsed_version =
                    InputFormatHandler::parse_version_string(tag_version, &args.input_format)?;
                // Validation passed - the tag is in a valid format
            }

            // Apply VCS overrides (including --tag-version with input format validation)
            vcs_data = VcsOverrideProcessor::apply_overrides(vcs_data, &args)?;

            // Convert VCS data to ZervVars
            let vars = vcs_data_to_zerv_vars(vcs_data)?;

            // Create Zerv object from vars and schema
            create_zerv_version(vars, args.schema.as_deref(), args.schema_ron.as_deref())?
        }
        "stdin" => {
            // Parse stdin as Zerv RON (input_format must be "zerv" for stdin)
            let mut zerv_from_stdin = InputFormatHandler::parse_stdin(&args.input_format)?;

            // Apply overrides to the parsed Zerv object if any are specified
            if args.has_overrides() {
                // Convert Zerv back to VcsData-like structure for override processing
                let mut temp_vcs_data = zerv_to_vcs_data(&zerv_from_stdin)?;

                // Apply overrides
                temp_vcs_data = VcsOverrideProcessor::apply_overrides(temp_vcs_data, &args)?;

                // Convert back to ZervVars and create new Zerv object
                let updated_vars = vcs_data_to_zerv_vars(temp_vcs_data)?;

                // Preserve the original schema but update vars
                zerv_from_stdin.vars = updated_vars;
            }

            zerv_from_stdin
        }
        source => return Err(ZervError::UnknownSource(source.to_string())),
    };

    // 3. Apply schema if specified (this can override the schema from stdin)
    if args.schema.is_some() || args.schema_ron.is_some() {
        zerv_object = create_zerv_version(
            zerv_object.vars,
            args.schema.as_deref(),
            args.schema_ron.as_deref(),
        )?;
    }

    // 4. Apply output formatting with enhanced options
    let output = OutputFormatter::format_output(
        &zerv_object,
        &args.output_format,
        args.output_prefix.as_deref(),
        args.output_template.as_deref(),
    )?;

    Ok(output)
}

/// Convert Zerv object back to VcsData for override processing
/// This is a helper function to enable override application on stdin input
fn zerv_to_vcs_data(zerv: &Zerv) -> Result<crate::vcs::VcsData, ZervError> {
    use crate::vcs::VcsData;

    // Extract values from ZervVars, providing defaults where needed
    let vars = &zerv.vars;

    // Reconstruct tag version from major.minor.patch if available
    let tag_version = match (vars.major, vars.minor, vars.patch) {
        (Some(major), Some(minor), Some(patch)) => Some(format!("{major}.{minor}.{patch}")),
        _ => None,
    };

    Ok(VcsData {
        tag_version,
        distance: vars.distance.unwrap_or(0) as u32,
        commit_hash: vars
            .current_commit_hash
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        commit_hash_short: vars
            .current_commit_hash
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
            .chars()
            .take(7)
            .collect(),
        current_branch: vars.current_branch.clone(),
        commit_timestamp: vars.dev.unwrap_or(0) as i64,
        tag_timestamp: vars.tag_timestamp.map(|t| t as i64),
        is_dirty: vars.dirty.unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{FORMAT_PEP440, FORMAT_SEMVER, FORMAT_ZERV, SCHEMA_ZERV_STANDARD};
    use crate::test_utils::{GitRepoFixture, VersionTestUtils, should_run_docker_tests};
    use clap::Parser;
    use rstest::rstest;

    #[test]
    fn test_version_args_defaults() {
        let args = VersionArgs::try_parse_from(["zerv"]).unwrap();
        assert!(args.version.is_none());
        assert_eq!(args.source, "git");
        assert!(args.schema.is_none());
        assert!(args.schema_ron.is_none());
        assert_eq!(args.input_format, "auto");
        assert_eq!(args.output_format, FORMAT_SEMVER);

        // VCS override options should be None/false by default
        assert!(args.tag_version.is_none());
        assert!(args.distance.is_none());
        assert!(args.dirty.is_none());
        assert!(!args.clean);
        assert!(args.current_branch.is_none());
        assert!(args.commit_hash.is_none());

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
            "true",
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
        assert!(args.dirty.is_some());
        assert!(args.dirty.unwrap().value());
        assert!(!args.clean);
        assert_eq!(args.current_branch, Some("feature/test".to_string()));
        assert_eq!(args.commit_hash, Some("abc123".to_string()));
        assert_eq!(args.input_format, "semver");
        assert_eq!(args.output_prefix, Some("version:".to_string()));
    }

    #[test]
    fn test_version_args_clean_flag() {
        let args = VersionArgs::try_parse_from(["zerv", "--clean"]).unwrap();

        assert!(args.clean);
        assert!(args.distance.is_none());
        assert!(args.dirty.is_none());
    }

    #[test]
    fn test_version_args_fuzzy_bool_parsing() {
        // Test various fuzzy bool values
        let test_cases = [
            ("true", true),
            ("false", false),
            ("yes", true),
            ("no", false),
            ("1", true),
            ("0", false),
            ("on", true),
            ("off", false),
        ];

        for (input, expected) in &test_cases {
            let args = VersionArgs::try_parse_from(["zerv", "--dirty", input]).unwrap();

            assert!(args.dirty.is_some(), "Failed to parse '{input}'");
            assert_eq!(
                args.dirty.unwrap().value(),
                *expected,
                "Wrong value for '{input}'"
            );
        }
    }

    #[rstest]
    #[case("tagged_clean", "v1.0.0", 0, None, "1.0.0")]
    #[case("tagged_with_distance_1", "v1.0.0", 1, None, "1.0.0+main.<commit>")]
    #[case("tagged_with_distance_3", "v2.1.0", 3, None, "2.1.0+main.<commit>")]
    #[case("tagged_on_branch", "v1.5.0", 0, Some("feature"), "1.5.0")]
    #[case(
        "tagged_with_distance_on_branch",
        "v2.0.0",
        2,
        Some("dev"),
        "2.0.0+dev.<commit>"
    )]
    fn test_run_version_pipeline_with_docker_git(
        #[case] scenario: &str,
        #[case] tag: &str,
        #[case] commits_after_tag: u32,
        #[case] branch: Option<&str>,
        #[case] expected_version: &str,
    ) {
        if !should_run_docker_tests() {
            return;
        }

        // Create appropriate fixture based on commits_after_tag
        let fixture = if commits_after_tag == 0 {
            GitRepoFixture::tagged(tag).expect("Failed to create tagged repo")
        } else {
            GitRepoFixture::with_distance(tag, commits_after_tag)
                .expect("Failed to create repo with distance")
        };

        // Create branch if specified (after fixture creation)
        if let Some(branch_name) = branch {
            fixture
                .git_impl
                .create_branch(&fixture.test_dir, branch_name)
                .expect("Failed to create branch");
        }

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: None,
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        let version = result.unwrap_or_else(|_| panic!("Pipeline should succeed for {scenario}"));
        println!("Scenario {scenario}: Generated version: {version}");

        if expected_version.contains("<commit>") {
            VersionTestUtils::assert_version_pattern(&version, expected_version, scenario);
        } else {
            VersionTestUtils::assert_exact_version(&version, expected_version, scenario);
        }
    }

    #[test]
    fn test_run_version_pipeline_unknown_format() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: "unknown".to_string(),
            tag_version: None,
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(result.is_err(), "Pipeline should fail for unknown format");
        assert!(matches!(result, Err(ZervError::UnknownFormat(_))));
    }

    #[test]
    fn test_run_version_pipeline_with_overrides() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: Some("v2.0.0".to_string()),
            distance: Some(5),
            dirty: Some(FuzzyBool::new(true)),
            clean: false,
            current_branch: Some("feature/test".to_string()),
            commit_hash: Some("abc123def456".to_string()),
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(result.is_ok(), "Pipeline should succeed with overrides");

        let version = result.unwrap();
        // Should reflect the overridden values (v2.0.0 with distance 5 and dirty state)
        assert!(
            version.contains("2.0.0"),
            "Version should contain overridden major version"
        );
    }

    #[test]
    fn test_run_version_pipeline_with_clean_flag() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::with_distance("v1.0.0", 5)
            .expect("Failed to create repo with distance");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: None,
            distance: None,
            dirty: None,
            clean: true, // This should force clean state
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(result.is_ok(), "Pipeline should succeed with clean flag");

        let version = result.unwrap();
        // Should be clean version without distance/dirty indicators
        assert_eq!(version, "1.0.0", "Clean flag should produce clean version");
    }

    #[test]
    fn test_run_version_pipeline_with_output_prefix() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: None,
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: Some("version:".to_string()),
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(result.is_ok(), "Pipeline should succeed with output prefix");

        let version = result.unwrap();
        assert!(version.starts_with("version:"), "Output should have prefix");
        assert!(version.contains("1.0.0"), "Output should contain version");
    }

    #[test]
    fn test_run_version_pipeline_unknown_source() {
        let args = VersionArgs {
            version: None,
            source: "unknown".to_string(),
            schema: None,
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: None,
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: None,
        };

        let result = run_version_pipeline(args);
        assert!(result.is_err(), "Pipeline should fail for unknown source");
        assert!(matches!(result, Err(ZervError::UnknownSource(_))));
    }

    #[test]
    fn test_run_version_pipeline_input_format_validation() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        // Test with invalid tag version format
        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "semver".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: Some("invalid-version".to_string()),
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(
            result.is_err(),
            "Pipeline should fail for invalid tag version"
        );
        assert!(matches!(result, Err(ZervError::InvalidVersion(_))));
    }

    #[test]
    fn test_run_version_pipeline_conflicting_overrides() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        // Test conflicting --clean with --distance
        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: None,
            distance: Some(5),
            dirty: None,
            clean: true, // Conflicts with distance
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(
            result.is_err(),
            "Pipeline should fail for conflicting options"
        );
        assert!(matches!(result, Err(ZervError::ConflictingOptions(_))));
    }

    #[test]
    fn test_run_version_pipeline_different_input_formats() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        // Test SemVer input format
        let args_semver = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "semver".to_string(),
            output_format: FORMAT_SEMVER.to_string(),
            tag_version: Some("2.0.0-alpha.1".to_string()),
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args_semver);
        assert!(
            result.is_ok(),
            "Pipeline should succeed with SemVer input format"
        );

        // Test PEP440 input format
        let args_pep440 = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "pep440".to_string(),
            output_format: FORMAT_PEP440.to_string(),
            tag_version: Some("2.0.0a1".to_string()),
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args_pep440);
        assert!(
            result.is_ok(),
            "Pipeline should succeed with PEP440 input format"
        );
    }

    #[test]
    fn test_run_version_pipeline_zerv_output_format() {
        if !should_run_docker_tests() {
            return;
        }

        let fixture = GitRepoFixture::tagged("v1.0.0").expect("Failed to create tagged repo");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            input_format: "auto".to_string(),
            output_format: FORMAT_ZERV.to_string(),
            tag_version: None,
            distance: None,
            dirty: None,
            clean: false,
            current_branch: None,
            commit_hash: None,
            output_template: None,
            output_prefix: None,
            directory: Some(fixture.path().to_str().unwrap().to_string()),
        };

        let result = run_version_pipeline(args);
        assert!(
            result.is_ok(),
            "Pipeline should succeed with Zerv output format"
        );

        let output = result.unwrap();
        // Zerv RON output should contain schema and vars
        assert!(
            output.contains("schema"),
            "Zerv output should contain schema"
        );
        assert!(output.contains("vars"), "Zerv output should contain vars");
    }
}
