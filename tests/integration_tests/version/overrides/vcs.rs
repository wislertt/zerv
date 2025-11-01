use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[fixture]
fn clean_fixture() -> ZervFixture {
    ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
        Some(0),
        Some(false),
        None,
        None,
        None,
        None,
        None,
    )
}

#[fixture]
fn fixture_with_distance() -> ZervFixture {
    ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
        Some(10),
        Some(false),
        None,
        None,
        None,
        None,
        None,
    )
}

#[fixture]
fn dirty_fixture() -> ZervFixture {
    ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
        Some(0),
        Some(true),
        None,
        None,
        None,
        None,
        None,
    )
}

#[fixture]
fn fixture_with_branch() -> ZervFixture {
    ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
        Some(0),
        Some(false),
        None,
        None,
        None,
        None,
        Some("old-branch".to_string()),
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
        clean_fixture: ZervFixture,
        #[case] tag_version: &str,
        #[case] input_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = clean_fixture.build().to_string();

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

    #[rstest]
    fn test_tag_version_replaces_stdin_version(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 5.0.0 \
             --input-format semver --output-format semver",
            zerv_ron,
        );

        assert_eq!(output, "5.0.0");
    }

    #[rstest]
    fn test_tag_version_with_v_prefix(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version v2.0.0 \
             --input-format semver --output-format semver",
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
    fn test_distance_override_basic(clean_fixture: ZervFixture, #[case] distance: u32) {
        let zerv_ron = clean_fixture.build().to_string();

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

    #[rstest]
    fn test_distance_override_affects_tier(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.with_standard_tier_3().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --distance 5 --output-format pep440",
            zerv_ron,
        );

        assert_eq!(
            output, "1.0.0+5",
            "Schema from stdin is tier 3, so includes both post and distance in build"
        );
    }

    #[rstest]
    fn test_distance_replaces_stdin_distance(fixture_with_distance: ZervFixture) {
        let zerv_ron = fixture_with_distance.build().to_string();

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

    #[rstest]
    fn test_dirty_flag_sets_dirty_true(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

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

    #[rstest]
    fn test_no_dirty_flag_sets_dirty_false(dirty_fixture: ZervFixture) {
        let zerv_ron = dirty_fixture.build().to_string();

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

    #[rstest]
    fn test_dirty_replaces_stdin_dirty(dirty_fixture: ZervFixture) {
        let zerv_ron = dirty_fixture.build().to_string();

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

    #[rstest]
    fn test_clean_sets_distance_zero_dirty_false(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture
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

    #[rstest]
    fn test_clean_forces_tier_1(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture
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

    #[rstest]
    fn test_clean_overrides_dirty_stdin(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture
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

    #[rstest]
    fn test_bumped_branch_override_basic(fixture_with_branch: ZervFixture) {
        let zerv_ron = fixture_with_branch.build().to_string();

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

    #[rstest]
    fn test_bumped_branch_in_template(fixture_with_branch: ZervFixture) {
        let zerv_ron = fixture_with_branch.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch dev --output-template {{bumped_branch}}",
            zerv_ron,
        );

        assert_eq!(output, "dev");
    }

    #[rstest]
    fn test_bumped_branch_with_special_chars(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let cmd = format!(
            r#"version --source stdin --bumped-branch "{}" --output-template "{}""#,
            "feature/USER-123", "{{bumped_branch}}"
        );
        let output = TestCommand::run_with_stdin(&cmd, zerv_ron);

        assert_eq!(output, "feature/USER-123");
    }
}

mod bumped_commit_hash_override {
    use super::*;

    #[rstest]
    fn test_bumped_commit_hash_override_full(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();
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

    #[rstest]
    fn test_bumped_commit_hash_override_short(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();
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

    #[rstest]
    fn test_bumped_commit_hash_in_template(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-commit-hash deadbeef \
             --output-template {{bumped_commit_hash}}",
            zerv_ron,
        );

        assert_eq!(output, "deadbeef");
    }
}

mod bumped_timestamp_override {
    use super::*;

    #[rstest]
    #[case::zerv_format(1672531200, "--output-format zerv", |output: &str, timestamp: i64| {
        output.contains(&format!("bumped_timestamp: Some({timestamp})"))
    })]
    #[case::zerv_large(1704067200, "--output-format zerv", |output: &str, timestamp: i64| {
        output.contains(&format!("bumped_timestamp: Some({timestamp})"))
    })]
    #[case::template_basic(
        1672531200,
        "--output-template {{bumped_timestamp}}",
        |output: &str, timestamp: i64| {
        output == timestamp.to_string()
    })]
    #[case::template_formatted(
        1672531200,
        r#"--output-template "{{ format_timestamp(value=bumped_timestamp, format=\"compact_date\") }}""#,
        |output: &str, _: i64| {
            output == "20230101"
        }
    )]
    fn test_bumped_timestamp_override(
        clean_fixture: ZervFixture,
        #[case] timestamp: i64,
        #[case] output_option: &str,
        #[case] assertion: impl Fn(&str, i64) -> bool,
    ) {
        let zerv_ron = clean_fixture.build().to_string();

        let command =
            format!("version --source stdin --bumped-timestamp {timestamp} {output_option}");
        let output = TestCommand::run_with_stdin(&command, zerv_ron);

        assert!(
            assertion(&output, timestamp),
            "Expected timestamp assertion to pass for timestamp {timestamp}, got: {}",
            output
        );
    }
}

mod vcs_overrides_combined {
    use super::*;

    #[rstest]
    #[ignore]
    fn test_tag_version_and_distance(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.with_standard_tier_3().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 2.0.0 \
             --input-format semver --distance 5 --output-format pep440",
            zerv_ron,
        );

        assert!(
            output.contains("2.0.0+5"),
            "Expected tag override (2.0.0) with distance (5), got: {}",
            output
        );
    }

    #[rstest]
    fn test_distance_and_dirty(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

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

    #[rstest]
    fn test_branch_and_commit(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch dev --bumped-commit-hash abc123 \
             --output-template {{bumped_branch}}-{{bumped_commit_hash}}",
            zerv_ron,
        );

        assert_eq!(output, "dev-abc123");
    }

    #[rstest]
    fn test_all_vcs_overrides(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 3.0.0 \
             --input-format semver --distance 10 --dirty \
             --bumped-branch feature --bumped-commit-hash xyz789 --output-format zerv",
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

    #[rstest]
    #[case::dirty_and_no_dirty("--dirty --no-dirty", "--dirty", "--no-dirty")]
    #[case::clean_and_distance("--clean --distance 5", "--clean", "--distance")]
    #[case::clean_and_dirty("--clean --dirty", "--clean", "--dirty")]
    #[case::clean_and_no_dirty("--clean --no-dirty", "--clean", "--no-dirty")]
    fn test_conflicting_flags(
        clean_fixture: ZervFixture,
        #[case] args: &str,
        #[case] flag1: &str,
        #[case] flag2: &str,
    ) {
        let zerv_ron = clean_fixture.build().to_string();

        let result = TestCommand::run_with_stdin_expect_fail(
            &format!("version --source stdin {args}"),
            zerv_ron,
        );
        assert!(
            result.contains(flag1) && result.contains(flag2),
            "Expected conflict error mentioning both flags, got: {result}"
        );
    }
}

mod vcs_override_edge_cases {
    use super::*;

    #[rstest]
    fn test_distance_zero_explicit(fixture_with_distance: ZervFixture) {
        let zerv_ron = fixture_with_distance.build().to_string();

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

    #[rstest]
    fn test_branch_empty_string(fixture_with_branch: ZervFixture) {
        let zerv_ron = fixture_with_branch.build().to_string();

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

    #[rstest]
    fn test_tag_version_without_input_format_semver(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --tag-version 2.0.0 --output-format semver",
            zerv_ron,
        );

        assert_eq!(
            output, "2.0.0",
            "Expected tag override with auto-detected format"
        );
    }

    #[rstest]
    fn test_multiple_overrides_preserve_original_tier(clean_fixture: ZervFixture) {
        let zerv_ron = clean_fixture
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
