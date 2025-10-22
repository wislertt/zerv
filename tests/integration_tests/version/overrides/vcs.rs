use rstest::rstest;
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

fn create_clean_fixture(version: (u64, u64, u64)) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_vcs_data(Some(0), Some(false), None, None, None, None, None)
}

fn create_fixture_with_distance(version: (u64, u64, u64), distance: u64) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_vcs_data(Some(distance), Some(false), None, None, None, None, None)
}

fn create_dirty_fixture(version: (u64, u64, u64)) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_vcs_data(Some(0), Some(true), None, None, None, None, None)
}

fn create_fixture_with_branch(version: (u64, u64, u64), branch: &str) -> ZervFixture {
    ZervFixture::new()
        .with_version(version.0, version.1, version.2)
        .with_vcs_data(
            Some(0),
            Some(false),
            None,
            None,
            None,
            None,
            Some(branch.to_string()),
        )
}

mod tag_version_override {
    use super::*;

    #[rstest]
    #[case::semver_basic("1.0.0", "semver", "1.0.0")]
    #[case::semver_prerelease("2.0.0-beta.1", "semver", "2.0.0-beta.1")]
    #[case::pep440_basic("3.1.4", "pep440", "3.1.4")]
    #[case::pep440_prerelease("1.0.0a1", "pep440", "1.0.0-alpha.1")]
    fn test_tag_version_override(
        #[case] tag_version: &str,
        #[case] input_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = create_clean_fixture((0, 0, 1)).build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --tag-version {tag_version} --input-format {input_format}"
            ),
            zerv_ron,
        );

        assert!(
            output.contains(expected),
            "Expected output to contain '{}', got: {}",
            expected,
            output
        );
    }

    #[test]
    fn test_tag_version_replaces_stdin_version() {
        let zerv_ron = create_clean_fixture((1, 2, 3)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 5.0.0 --input-format semver --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "5.0.0");
    }

    #[test]
    fn test_tag_version_with_v_prefix() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version v2.0.0 --input-format semver --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "2.0.0");
    }
}

mod distance_override {
    use super::*;

    #[rstest]
    #[case::zero(0)]
    #[case::small(5)]
    #[case::large(100)]
    fn test_distance_override_basic(#[case] distance: u32) {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            &format!("version --source stdin --distance {distance} --output-format zerv"),
            zerv_ron,
        );

        assert!(
            output.contains(&format!("distance: Some({distance})")),
            "Expected distance: Some({distance}) in output: {}",
            output
        );
    }

    #[test]
    fn test_distance_override_affects_tier() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_standard_tier_3()
            .with_vcs_data(Some(0), Some(false), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --distance 5 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(
            output, "1.0.0+5",
            "Schema from stdin is tier 3, so includes both post and distance in build"
        );
    }

    #[test]
    fn test_distance_replaces_stdin_distance() {
        let zerv_ron = create_fixture_with_distance((1, 0, 0), 10)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --distance 3 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("distance: Some(3)"),
            "Expected overridden distance: Some(3), got: {}",
            output
        );
        assert!(
            !output.contains("distance: Some(10)"),
            "Should not contain original distance: Some(10), got: {}",
            output
        );
    }
}

mod dirty_override {
    use super::*;

    #[test]
    fn test_dirty_flag_sets_dirty_true() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --dirty --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("dirty: Some(true)"),
            "Expected dirty: Some(true) in output: {}",
            output
        );
    }

    #[test]
    fn test_no_dirty_flag_sets_dirty_false() {
        let zerv_ron = create_dirty_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --no-dirty --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("dirty: Some(false)"),
            "Expected dirty: Some(false) in output: {}",
            output
        );
    }

    #[test]
    fn test_dirty_replaces_stdin_dirty() {
        let zerv_ron = create_dirty_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --no-dirty --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("dirty: Some(false)"),
            "Expected overridden dirty: Some(false), got: {}",
            output
        );
    }
}

mod clean_override {
    use super::*;

    #[test]
    fn test_clean_sets_distance_zero_dirty_false() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_vcs_data(Some(5), Some(true), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --clean --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("distance: None"),
            "Expected distance: None with --clean, got: {}",
            output
        );
        assert!(
            output.contains("dirty: Some(false)"),
            "Expected dirty: Some(false) with --clean, got: {}",
            output
        );
    }

    #[test]
    fn test_clean_forces_tier_1() {
        let zerv_ron = ZervFixture::new()
            .with_version(2, 0, 0)
            .with_standard_tier_2()
            .with_vcs_data(Some(10), Some(false), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --clean --output-format pep440",
            zerv_ron,
        );
        assert_eq!(
            output, "2.0.0",
            "Expected tier 1 format (2.0.0) with --clean"
        );
    }

    #[test]
    fn test_clean_overrides_dirty_stdin() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 5, 2)
            .with_vcs_data(Some(3), Some(true), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --clean --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "1.5.2",
            "Expected clean release version with --clean"
        );
    }
}

mod bumped_branch_override {
    use super::*;

    #[test]
    fn test_bumped_branch_override_basic() {
        let zerv_ron = create_fixture_with_branch((1, 0, 0), "old-branch")
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch feature/new --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("bumped_branch: Some(\"feature/new\")"),
            "Expected bumped_branch override, got: {}",
            output
        );
    }

    #[test]
    fn test_bumped_branch_in_template() {
        let zerv_ron = create_fixture_with_branch((1, 0, 0), "main")
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch dev --output-template {{bumped_branch}}",
            zerv_ron,
        );

        assert_eq!(output, "dev");
    }

    #[test]
    fn test_bumped_branch_with_special_chars() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --bumped-branch "feature/USER-123" --output-template {{bumped_branch}}"#,
            zerv_ron,
        );

        assert_eq!(output, "feature/USER-123");
    }
}

mod bumped_commit_hash_override {
    use super::*;

    #[test]
    fn test_bumped_commit_hash_override_full() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();
        let full_hash = "abcdef1234567890abcdef1234567890abcdef12";

        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --bumped-commit-hash {full_hash} --output-format zerv"
            ),
            zerv_ron,
        );

        let expected_pattern = format!("bumped_commit_hash: Some(\"{}\")", full_hash);
        assert!(
            output.contains(&expected_pattern),
            "Expected bumped_commit_hash with full hash, got: {}",
            output
        );
    }

    #[test]
    fn test_bumped_commit_hash_override_short() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();
        let short_hash = "abc123";

        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --bumped-commit-hash {short_hash} --output-format zerv"
            ),
            zerv_ron,
        );

        assert!(
            output.contains(&format!("bumped_commit_hash: Some(\"{short_hash}\")")),
            "Expected short bumped_commit_hash override, got: {}",
            output
        );
    }

    #[test]
    fn test_bumped_commit_hash_in_template() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-commit-hash deadbeef --output-template {{bumped_commit_hash}}",
            zerv_ron,
        );

        assert_eq!(output, "deadbeef");
    }
}

mod vcs_overrides_combined {
    use super::*;

    #[test]
    #[ignore]
    fn test_tag_version_and_distance() {
        let zerv_ron = create_clean_fixture((1, 0, 0))
            .with_standard_tier_3()
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 2.0.0 --input-format semver --distance 5 --output-format pep440",
            zerv_ron,
        );

        assert!(
            output.contains("2.0.0+5"),
            "Expected tag override (2.0.0) with distance (5), got: {}",
            output
        );
    }

    #[test]
    fn test_distance_and_dirty() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --distance 3 --dirty --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("distance: Some(3)"),
            "Expected distance: Some(3), got: {}",
            output
        );
        assert!(
            output.contains("dirty: Some(true)"),
            "Expected dirty: Some(true), got: {}",
            output
        );
    }

    #[test]
    fn test_branch_and_commit() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch dev --bumped-commit-hash abc123 --output-template {{bumped_branch}}-{{bumped_commit_hash}}",
            zerv_ron,
        );

        assert_eq!(output, "dev-abc123");
    }

    #[test]
    fn test_all_vcs_overrides() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 3.0.0 --input-format semver --distance 10 --dirty --bumped-branch feature --bumped-commit-hash xyz789 --output-format zerv",
            zerv_ron,
        );

        assert!(output.contains("distance: Some(10)"));
        assert!(output.contains("dirty: Some(true)"));
        assert!(output.contains("bumped_branch: Some(\"feature\")"));
        assert!(output.contains("bumped_commit_hash: Some(\"xyz789\")"));
    }
}

mod vcs_override_conflicts {
    use super::*;

    #[test]
    fn test_dirty_and_no_dirty_conflict() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --dirty --no-dirty")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("--dirty") && stderr.contains("--no-dirty"),
            "Expected conflict error mentioning both flags, got: {stderr}"
        );
    }

    #[test]
    fn test_clean_and_distance_conflict() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --clean --distance 5")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("--clean") && stderr.contains("--distance"),
            "Expected conflict error mentioning both flags, got: {stderr}"
        );
    }

    #[test]
    fn test_clean_and_dirty_conflict() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --clean --dirty")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("--clean") && stderr.contains("--dirty"),
            "Expected conflict error mentioning both flags, got: {stderr}"
        );
    }

    #[test]
    fn test_clean_and_no_dirty_conflict() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --clean --no-dirty")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("--clean") && stderr.contains("--no-dirty"),
            "Expected conflict error mentioning both flags, got: {stderr}"
        );
    }
}

mod vcs_override_edge_cases {
    use super::*;

    #[test]
    fn test_distance_zero_explicit() {
        let zerv_ron = create_fixture_with_distance((1, 0, 0), 5)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --distance 0 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("distance: Some(0)"),
            "Expected explicit distance: Some(0) override, got: {}",
            output
        );
    }

    #[test]
    fn test_branch_empty_string() {
        let zerv_ron = create_fixture_with_branch((1, 0, 0), "main")
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --bumped-branch "" --output-format zerv"#,
            zerv_ron,
        );

        assert!(
            output.contains("bumped_branch: Some(\"\")"),
            "Expected empty bumped_branch override, got: {}",
            output
        );
    }

    #[test]
    fn test_tag_version_without_input_format_semver() {
        let zerv_ron = create_clean_fixture((1, 0, 0)).build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 2.0.0 --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "2.0.0",
            "Expected tag override with auto-detected format"
        );
    }

    #[test]
    fn test_multiple_overrides_preserve_original_tier() {
        let zerv_ron = ZervFixture::new()
            .with_version(2024, 12, 1)
            .with_calver_tier_2()
            .with_vcs_data(
                Some(1),
                Some(false),
                None,
                None,
                None,
                None,
                Some("main".to_string()),
            )
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --distance 5 --output-format zerv",
            zerv_ron,
        );

        assert!(
            output.contains("distance: Some(5)"),
            "Expected distance: Some(5) override, got: {}",
            output
        );
    }
}
