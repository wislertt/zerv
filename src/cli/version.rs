use crate::constants::{FORMAT_PEP440, FORMAT_SEMVER, SUPPORTED_FORMATS};
use crate::error::ZervError;
use crate::pipeline::vcs_data_to_zerv_vars;
use crate::schema::create_zerv_version;
use crate::vcs::detect_vcs;
use crate::version::pep440::PEP440;
use crate::version::semver::SemVer;
use clap::Parser;
use std::env::current_dir;

#[derive(Parser)]
pub struct VersionArgs {
    /// Version string (when using string source)
    pub version: Option<String>,

    /// Source type
    #[arg(long, default_value = "git")]
    pub source: String,

    /// Schema preset name
    #[arg(long)]
    pub schema: Option<String>,

    /// Custom RON schema
    #[arg(long)]
    pub schema_ron: Option<String>,

    /// Output format
    #[arg(long, default_value = FORMAT_SEMVER)]
    pub output_format: String,
}

pub fn run_version_pipeline(args: VersionArgs) -> Result<String, ZervError> {
    // 1. Get VCS data
    let vcs_data = detect_vcs(&current_dir()?)?.get_vcs_data()?;

    // 2. Convert to ZervVars
    let vars = vcs_data_to_zerv_vars(vcs_data)?;

    // 3. Create Zerv version object from vars and schema
    let zerv = create_zerv_version(vars, args.schema.as_deref(), args.schema_ron.as_deref())?;

    // 4. Apply output format
    match args.output_format.as_str() {
        FORMAT_PEP440 => Ok(PEP440::from(zerv).to_string()),
        FORMAT_SEMVER => Ok(SemVer::from(zerv).to_string()),
        format => {
            eprintln!("Unknown output format: {format}");
            eprintln!("Supported formats: {}", SUPPORTED_FORMATS.join(", "));
            Err(ZervError::UnknownFormat(format.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::SCHEMA_ZERV_STANDARD;
    use crate::test_utils::{GitRepoFixture, VersionTestUtils, should_run_docker_tests};
    use clap::Parser;
    use rstest::rstest;
    use std::env;

    #[test]
    fn test_version_args_defaults() {
        let args = VersionArgs::try_parse_from(["zerv"]).unwrap();
        assert!(args.version.is_none());
        assert_eq!(args.source, "git");
        assert!(args.schema.is_none());
        assert!(args.schema_ron.is_none());
        assert_eq!(args.output_format, FORMAT_SEMVER);
    }

    // TODO: XXXXXXXXXXX
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

        let original_dir = env::current_dir().expect("Failed to get current dir");
        env::set_current_dir(fixture.path()).expect("Failed to change dir");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            output_format: FORMAT_SEMVER.to_string(),
        };

        let result = run_version_pipeline(args);

        // Restore directory (ignore errors if original dir was deleted)
        let _ = env::set_current_dir(&original_dir);

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

        let original_dir = env::current_dir().expect("Failed to get current dir");
        env::set_current_dir(fixture.path()).expect("Failed to change dir");

        let args = VersionArgs {
            version: None,
            source: "git".to_string(),
            schema: Some(SCHEMA_ZERV_STANDARD.to_string()),
            schema_ron: None,
            output_format: "unknown".to_string(),
        };

        let result = run_version_pipeline(args);
        env::set_current_dir(original_dir).expect("Failed to restore dir");

        assert!(result.is_err(), "Pipeline should fail for unknown format");
        assert!(matches!(result, Err(ZervError::UnknownFormat(_))));
    }
}
