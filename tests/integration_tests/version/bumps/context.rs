//! Context bump tests
//!
//! Tests for --bump-context and --no-bump-context options.
//! These tests verify that context control works correctly for determining
//! whether VCS context (distance, dirty) is included in the bumped version.

use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::formats::{
    PEP440,
    SEMVER,
};
use zerv::version::PreReleaseLabel;

use super::{
    base_zerv_fixture,
    zerv_with_vcs_fixture,
};
use crate::util::TestCommand;

/// Zerv fixture with VCS data for context tests
#[fixture]
fn context_bump_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(5)
        .with_dirty(true)
}

/// Zerv fixture with clean VCS state
#[fixture]
fn clean_context_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(3) // clean state distance
}

/// Zerv fixture without VCS data (pure tag version)
#[fixture]
fn pure_tag_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
    // No distance - represents a clean tag
}

mod bump_context {
    use super::*;

    #[rstest]
    #[case("2.0.0+5", SEMVER)] // Default behavior includes VCS context
    #[case("2.0.0+5", PEP440)] // PEP440 format
    fn test_bump_context_default_behavior(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-format {} --bump-major",
                format
            ),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_bump_context_explicit_flag(context_bump_fixture: ZervFixture) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format zerv --bump-major --bump-context",
            zerv_ron,
        );

        // Check that VCS context is preserved
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(0)"));
        assert!(output.contains("patch: Some(0)"));
        assert!(output.contains("distance: Some(5)"));
        assert!(output.contains("dirty: Some(true)"));
        assert!(output.contains("bumped_timestamp: Some("));
    }

    #[rstest]
    #[case("2.0.0+3", SEMVER)] // Clean state only includes distance
    fn test_bump_context_clean_state(
        clean_context_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let zerv_ron = clean_context_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-format {} --bump-major",
                format
            ),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("2.0.0+d5.dirty")] // Context with template output
    fn test_context_with_template_output(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --bump-major --output-template "{{major}}.{{minor}}.{{patch}}{{#if prerelease}}-{{pre_release_label}}.{{pre_release_num}}{{/if}}{{#if distance}}+d{{distance}}{{/if}}{{#if dirty}}.dirty{{/if}}""#,
            zerv_ron,
        );

        assert_eq!(output, expected);
    }
}

mod no_bump_context {
    use super::*;

    #[rstest]
    #[case("2.0.0+0", SEMVER)] // No VCS context - distance reset to 0
    #[case("2.0.0+0", PEP440)] // PEP440 format
    fn test_no_bump_context_simple(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-format {} --bump-major --no-bump-context",
                format
            ),
            zerv_ron,
        );

        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_no_bump_context_zerv_format(context_bump_fixture: ZervFixture) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --output-format zerv --bump-major --no-bump-context",
            zerv_ron,
        );

        // Check that VCS context is cleared (distance=0, dirty=false)
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(0)"));
        assert!(output.contains("patch: Some(0)"));
        assert!(output.contains("distance: Some(0)"));
        assert!(output.contains("dirty: Some(false)"));
        assert!(!output.contains("bumped_timestamp: Some("));
    }

    #[rstest]
    #[case("2.0.0")] // No context with template output
    fn test_no_bump_context_with_template_output(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --bump-major --no-bump-context --output-template "{{major}}.{{minor}}.{{patch}}{{#if prerelease}}-{{pre_release_label}}.{{pre_release_num}}{{/if}}{{#if distance}}+d{{distance}}{{/if}}{{#if dirty}}.dirty{{/if}}""#,
            zerv_ron,
        );

        assert_eq!(output, expected);
    }
}

mod context_interactions {
    use super::*;

    #[rstest]
    fn test_bump_context_default_behavior_zerv(context_bump_fixture: ZervFixture) {
        let zerv_ron = context_bump_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --output-format zerv",
            zerv_ron,
        );

        // Default behavior preserves VCS context
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("distance: Some(5)"));
        assert!(output.contains("dirty: Some(true)"));
        assert!(output.contains("bumped_timestamp: Some("));
    }

    #[rstest]
    fn test_context_behavior_without_vcs_data(base_zerv_fixture: ZervFixture) {
        let zerv_ron = base_zerv_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --output-format zerv",
            zerv_ron,
        );

        // When no VCS data, distance and dirty are None
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("distance: None"));
        assert!(output.contains("dirty: None"));
    }

    #[rstest]
    fn test_context_preserves_other_version_data(zerv_with_vcs_fixture: ZervFixture) {
        let zerv_ron = zerv_with_vcs_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --no-bump-context --bump-major --output-format zerv",
            zerv_ron,
        );

        // Context flag doesn't affect version components
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(0)"));
        assert!(output.contains("patch: Some(0)"));
        // But clears VCS context
        assert!(output.contains("distance: Some(0)"));
        assert!(output.contains("dirty: Some(false)"));
    }
}
