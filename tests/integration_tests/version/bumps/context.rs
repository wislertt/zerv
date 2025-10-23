//! Context bump tests
//!
//! Tests for --bump-context and --no-bump-context options.
//! These tests verify that context control works correctly for determining
//! whether VCS context (distance, dirty) is included in the bumped version.

use rstest::{fixture, rstest};
use crate::util::TestCommand;
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::{
    sources::STDIN,
    formats::SEMVER,
    formats::PEP440,
    formats::ZERV,
};
use zerv::version::PreReleaseLabel;

use super::{base_zerv_fixture, zerv_with_vcs_fixture};

/// Zerv fixture with VCS data for context tests
#[fixture]
fn context_bump_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(5)
}

/// Zerv fixture with clean VCS state
#[fixture]
fn clean_context_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(3)  // clean state distance
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
    #[case("2.0.0-alpha.1+d5.dirty", ZERV)]  // Default behavior includes VCS context
    #[case("2.0.0", SEMVER)]                 // SemVer format
    #[case("2.0.0", PEP440)]                 // PEP440 format
    fn test_bump_context_default_behavior(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--output-format", format, "--bump-major"
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1+d5.dirty", ZERV)]  // Explicit --bump-context
    fn test_bump_context_explicit_flag(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--output-format", format,
            "--bump-major", "--bump-context"
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // Major bump without context
    #[case("1.3.0-alpha.1")]  // Minor bump without context
    #[case("1.2.4-alpha.1")]  // Patch bump without context
    fn test_bump_context_with_primary_bumps(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let bump_type = if expected.starts_with("2.") {
            "--bump-major"
        } else if expected.contains(".3.") {
            "--bump-patch"
        } else {
            "--bump-minor"
        };

        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--no-bump-context", bump_type, "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-alpha.1")]  // Epoch bump without context
    #[case("1.2.3-beta.0")]     // Label bump without context
    #[case("1.2.3-alpha.2")]    // Prerelease number bump without context
    fn test_bump_context_with_secondary_bumps(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let args = match expected {
            s if s.contains('!') => &["version", "--source", STDIN, "--no-bump-context", "--bump-epoch", "--output-format", ZERV],
            s if s.contains("beta") => &["version", "--source", STDIN, "--no-bump-context", "--bump-pre-release-label", "beta", "--output-format", ZERV],
            s if s.contains("alpha.2") => &["version", "--source", STDIN, "--no-bump-context", "--bump-pre-release-num", "--output-format", ZERV],
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(args, &input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.2.3-alpha.1")]  // Core bump without context
    fn test_bump_context_with_schema_bumps(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--no-bump-context", "--bump-core", "0", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0+d5.dirty")]  // Context with stable version (no prerelease)
    fn test_bump_context_with_stable_version(
        #[case] expected: &str,
    ) {
        // Create stable version with VCS data
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)

            .with_distance(
                Some("v1.2.3".to_string()),
                Some(5),
                Some(true),
            );

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1+d3")]  // Clean state only includes distance
    fn test_bump_context_clean_state(
        clean_context_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = clean_context_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}

mod no_bump_context {
    use super::*;

    #[rstest]
    #[case("2.0.0-alpha.1", ZERV)]  // No VCS context included
    #[case("2.0.0", SEMVER)]        // SemVer format
    #[case("2.0.0", PEP440)]        // PEP440 format
    fn test_no_bump_context_simple(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] format: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--output-format", format,
            "--bump-major", "--no-bump-context"
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // Pure tag version (no VCS data available)
    fn test_no_bump_context_pure_tag(
        pure_tag_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = pure_tag_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", "--no-bump-context", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.0.0-alpha.1")]  // Multiple bumps without context
    fn test_no_bump_context_multiple_bumps(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--no-bump-context",
            "--bump-major", "--bump-minor", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!1.2.3-alpha.1")]  // Secondary bump without context
    fn test_no_bump_context_secondary_bumps(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--no-bump-context",
            "--bump-epoch", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.2.3-alpha.1")]  // Schema bump without context
    fn test_no_bump_context_schema_bumps(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--no-bump-context",
            "--bump-core", "0", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}

mod context_interactions {
    use super::*;

    #[rstest]
    #[case("--bump-context", "--no-bump-context", "2.0.0-alpha.1")]  // no-bump-context wins
    #[case("--no-bump-context", "--bump-context", "2.0.0-alpha.1+d5.dirty")]  // bump-context wins
    fn test_context_flag_precedence(
        context_bump_fixture: ZervFixture,
        #[case] first_flag: &str,
        #[case] second_flag: &str,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", first_flag, second_flag, "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // Default behavior when no VCS data
    fn test_context_behavior_without_vcs_data(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = base_zerv_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // Explicit --no-bump-context when no VCS data
    fn test_no_bump_context_without_vcs_data(
        base_zerv_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = base_zerv_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", "--no-bump-context", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1+d5.dirty")]  // Context preserved with template output
    fn test_context_with_template_output(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major",
            "--output-template", "{{major}}.{{minor}}.{{patch}}{{#if prerelease}}-{{pre_release_label}}.{{pre_release_num}}{{/if}}{{#if distance}}+d{{distance}}{{/if}}{{#if dirty}}.dirty{{/if}}"
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // No context with template output
    fn test_no_bump_context_with_template_output(
        context_bump_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = context_bump_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--bump-major", "--no-bump-context",
            "--output-template", "{{major}}.{{minor}}.{{patch}}{{#if prerelease}}-{{pre_release_label}}.{{pre_release_num}}{{/if}}{{#if distance}}+d{{distance}}{{/if}}{{#if dirty}}.dirty{{/if}}"
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // Context flag doesn't affect other version data
    fn test_context_preserves_other_version_data(
        zerv_with_vcs_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = zerv_with_vcs_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN, "--no-bump-context", "--bump-major", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}
