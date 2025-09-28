use crate::cli::version::{args::VersionArgs, pipeline::run_version_pipeline};
use crate::constants::{formats, schema_names, sources};
use crate::error::ZervError;
use crate::test_utils::{
    GitRepoFixture, VersionArgsFixture, VersionTestUtils, should_run_docker_tests,
};
use clap::Parser;
use rstest::rstest;

#[test]
fn test_version_args_defaults() {
    let args = VersionArgs::try_parse_from(["zerv"]).unwrap();
    assert!(args.version.is_none());
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
    assert!(args.bump_distance.is_none());
    assert!(args.bump_post.is_none());
    assert!(args.bump_dev.is_none());
    assert!(args.bump_pre_release_num.is_none());
    assert!(args.bump_epoch.is_none());

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
    let args = VersionArgs::try_parse_from(["zerv", "--clean"]).unwrap();

    assert!(args.clean);
    assert!(args.distance.is_none());
    assert!(!args.dirty);
    assert!(!args.no_dirty);
}

#[test]
fn test_version_args_dirty_flags() {
    // Test --dirty flag
    let args = VersionArgs::try_parse_from(["zerv", "--dirty"]).unwrap();
    assert!(args.dirty);
    assert!(!args.no_dirty);

    // Test --no-dirty flag
    let args = VersionArgs::try_parse_from(["zerv", "--no-dirty"]).unwrap();
    assert!(!args.dirty);
    assert!(args.no_dirty);

    // Test both flags together should fail validation
    let args = VersionArgs::try_parse_from(["zerv", "--dirty", "--no-dirty"]).unwrap();
    assert!(args.dirty);
    assert!(args.no_dirty);

    // The conflict should be caught by VcsOverrideProcessor validation
    let vcs_data = crate::vcs::VcsData::default();
    let result =
        crate::cli::utils::vcs_override::VcsOverrideProcessor::apply_overrides(vcs_data, &args);
    assert!(result.is_err());
}

#[test]
fn test_version_args_fixture() {
    let args = VersionArgsFixture::create();
    assert_eq!(args.source, sources::GIT);
    assert_eq!(args.output_format, formats::SEMVER);

    let args_with_overrides = VersionArgsFixture::with_overrides();
    assert_eq!(args_with_overrides.tag_version, Some("v2.0.0".to_string()));
    assert_eq!(args_with_overrides.distance, Some(5));
    assert!(args_with_overrides.dirty);

    let args_with_clean = VersionArgsFixture::with_clean();
    assert!(args_with_clean.clean);

    let args_with_bumps = VersionArgsFixture::with_bumps();
    assert!(args_with_bumps.bump_major.is_some());
    assert!(args_with_bumps.bump_minor.is_some());
    assert!(args_with_bumps.bump_patch.is_some());
}

#[rstest]
#[case("tagged_clean", "v1.0.0", 0, None, "1.0.0")]
#[case(
    "tagged_with_distance_1",
    "v1.0.0",
    1,
    None,
    "1.0.0-post.1+main.<commit>"
)]
#[case(
    "tagged_with_distance_3",
    "v2.1.0",
    3,
    None,
    "2.1.0-post.3+main.<commit>"
)]
#[case("tagged_on_branch", "v1.5.0", 0, Some("feature"), "1.5.0")]
#[case(
    "tagged_with_distance_on_branch",
    "v2.0.0",
    2,
    Some("dev"),
    "2.0.0-post.2+dev.<commit>"
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

    let mut args = VersionArgsFixture::create();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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

    let mut args = VersionArgsFixture::create();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.output_format = "unknown".to_string();
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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

    let mut args = VersionArgsFixture::with_overrides();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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

    let fixture =
        GitRepoFixture::with_distance("v1.0.0", 5).expect("Failed to create repo with distance");

    let mut args = VersionArgsFixture::with_clean();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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

    let mut args = VersionArgsFixture::create();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.output_prefix = Some("version:".to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

    let result = run_version_pipeline(args);
    assert!(result.is_ok(), "Pipeline should succeed with output prefix");

    let version = result.unwrap();
    assert!(version.starts_with("version:"), "Output should have prefix");
    assert!(version.contains("1.0.0"), "Output should contain version");
}

#[test]
fn test_run_version_pipeline_unknown_source() {
    let mut args = VersionArgsFixture::create();
    args.source = "unknown".to_string();

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
    let mut args = VersionArgsFixture::create();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.input_format = formats::SEMVER.to_string();
    args.tag_version = Some("invalid-version".to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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
    let mut args = VersionArgsFixture::with_conflicts();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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
    let mut args_semver = VersionArgsFixture::create();
    args_semver.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args_semver.input_format = formats::SEMVER.to_string();
    args_semver.tag_version = Some("2.0.0-alpha.1".to_string());
    args_semver.directory = Some(fixture.path().to_str().unwrap().to_string());

    let result = run_version_pipeline(args_semver);
    assert!(
        result.is_ok(),
        "Pipeline should succeed with SemVer input format"
    );

    // Test PEP440 input format
    let mut args_pep440 = VersionArgsFixture::create();
    args_pep440.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args_pep440.input_format = formats::PEP440.to_string();
    args_pep440.tag_version = Some("2.0.0a1".to_string());
    args_pep440.directory = Some(fixture.path().to_str().unwrap().to_string());

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

    let mut args = VersionArgsFixture::with_output_format(formats::ZERV);
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

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

#[test]
fn test_run_version_pipeline_with_context_control() {
    if !should_run_docker_tests() {
        return;
    }

    let fixture =
        GitRepoFixture::with_distance("v1.0.0", 5).expect("Failed to create repo with distance");

    // Test with --no-bump-context (should produce clean version)
    let mut args = VersionArgsFixture::with_no_bump_context();
    args.schema = Some(schema_names::ZERV_STANDARD.to_string());
    args.directory = Some(fixture.path().to_str().unwrap().to_string());

    let result = run_version_pipeline(args);
    assert!(
        result.is_ok(),
        "Pipeline should succeed with no-bump-context"
    );

    let version = result.unwrap();
    // Should be clean version without distance/dirty indicators
    assert_eq!(
        version, "1.0.0",
        "No-bump-context should produce clean version"
    );
}
