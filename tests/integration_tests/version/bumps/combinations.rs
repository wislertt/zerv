//! Cross-category bump combination tests
//!
//! Tests for complex bump scenarios involving multiple categories:
//! - Primary + Secondary bump combinations
//! - Primary + Schema bump combinations
//! - Secondary + Schema bump combinations
//! - All category combinations
//! - Context behavior with complex combinations

use rstest::{fixture, rstest};
use crate::util::TestCommand;
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::{
    sources::STDIN,
    formats::SEMVER,
    formats::ZERV,
    schema_names::ZERV_STANDARD,
};
use zerv::version::zerv::PreReleaseLabel;

/// Base fixture for combination tests
#[fixture]
fn combination_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_standard_tier_1()
}

/// Complete fixture with all components for complex combinations
#[fixture]
fn full_combination_fixture() -> ZervFixture {
    combination_fixture()
        .with_epoch(1)
        .with_extra_core(&[0, 1])
        .with_build(&[2])

            .with_distance(
            Some("v1.2.3-alpha.1".to_string()),
            Some(5),
            Some(true),
        )
}

mod primary_secondary_combinations {
    use super::*;

    #[rstest]
    #[case("2.3.4-alpha.2")]  // major + minor + patch + prerelease num
    fn test_primary_secondary_simple(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-num", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.4.0-beta.0")]  // primary bumps + label change
    #[case("3.4.0-alpha.2")]  // primary bumps + prerelease num
    #[case("4!1.2.3-alpha.2")]  // epoch + primary + prerelease num
    fn test_primary_secondary_variants(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = combination_fixture.to_ron();
        let args = match expected {
            s if s.contains("beta") => &[
                "version", "--source", STDIN, "--bump-major", "--bump-minor",
                "--bump-pre-release-label", "beta", "--output-format", ZERV
            ],
            s if s.contains("4!") => &[
                "version", "--source", STDIN, "--bump-epoch", "--bump-major",
                "--bump-minor", "--bump-pre-release-num", "--output-format", ZERV
            ],
            s if s.contains("3.4") => &[
                "version", "--source", STDIN, "--bump-major", "--bump-minor",
                "--bump-pre-release-num", "--output-format", ZERV
            ],
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(args, &input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("5.7.0-alpha.2", "3", "4", "1")]  // Custom values for primary + prerelease num
    fn test_primary_secondary_custom_values(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] major_value: &str,
        #[case] minor_value: &str,
        #[case] prerelease_value: &str,
    ) {
        let input = combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", major_value, "--bump-minor", minor_value,
            "--bump-pre-release-num", prerelease_value, "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.3.4-alpha.2.dev.3.post.4")]  // All secondary with primary
    fn test_primary_all_secondary_combination(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-num", "--bump-post", "--bump-dev",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}

mod primary_schema_combinations {
    use super::*;

    #[rstest]
    #[case("2.3.4-alpha.1.1.1+1")]  // primary + extra-core + build
    fn test_primary_schema_simple(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0, 0])
            .with_build(&[0]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-extra-core", "0", "--bump-extra-core", "1",
            "--bump-build", "0", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.2.3-alpha.1")]  // major + core bump
    #[case("1.4.3-alpha.1")]  // minor + core bump
    #[case("1.2.5-alpha.1")]  // patch + core bump
    fn test_primary_core_combinations(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = combination_fixture.to_ron();
        let args = match expected {
            s if s.starts_with("3.") => &[
                "version", "--source", STDIN, "--bump-major", "--bump-core", "0", "--output-format", ZERV
            ],
            s if s.starts_with("1.4") => &[
                "version", "--source", STDIN, "--bump-minor", "--bump-core", "1", "--output-format", ZERV
            ],
            s if s.starts_with("1.2.5") => &[
                "version", "--source", STDIN, "--bump-patch", "--bump-core", "2", "--output-format", ZERV
            ],
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(args, &input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("4.6.8-alpha.1.2.3+4.5")]  // All primary + schema with custom values
    fn test_primary_schema_custom_values(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0, 0])
            .with_build(&[0, 0]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "3", "--bump-minor", "4", "--bump-patch", "5",
            "--bump-extra-core", "0=2", "--bump-extra-core", "1=3",
            "--bump-build", "0=4", "--bump-build", "1=5",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.2.3-alpha.1")]  // major + core bump preserves prerelease
    fn test_primary_schema_preserve_prerelease(
        combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-core", "0", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}

mod secondary_schema_combinations {
    use super::*;

    #[rstest]
    #[case("2!1.2.3-alpha.1.1.1+1")]  // epoch + extra-core + build
    fn test_secondary_schema_simple(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0, 0])
            .with_build(&[0]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-epoch", "--bump-extra-core", "0", "--bump-extra-core", "1",
            "--bump-build", "0", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-beta.0.1")]  // label + extra-core
    #[case("1.2.3-alpha.2.1")]  // prerelease num + extra-core
    #[case("1.2.3-alpha.1.dev.3+1")]  // dev + build
    fn test_secondary_schema_variants(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0])
            .with_build(&[0]);

        let input = fixture.to_ron();
        let args = match expected {
            s if s.contains("beta") => &[
                "version", "--source", STDIN, "--bump-pre-release-label", "beta",
                "--bump-extra-core", "0", "--output-format", ZERV
            ],
            s if s.contains("alpha.2") => &[
                "version", "--source", STDIN, "--bump-pre-release-num",
                "--bump-extra-core", "0", "--output-format", ZERV
            ],
            s if s.contains("dev.3") => &[
                "version", "--source", STDIN, "--bump-dev", "--bump-build", "0",
                "--output-format", ZERV
            ],
            _ => unreachable!("Unexpected expected value pattern"),
        };

        let output = TestCommand::run_with_stdin(args, &input);
        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3!1.2.3-beta.0.2.3+4.5")]  // All secondary + schema with custom values
    fn test_secondary_schema_complex(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0, 0])
            .with_build(&[0, 0]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-epoch", "2",
            "--bump-pre-release-label", "beta", "--bump-post", "--bump-dev",
            "--bump-extra-core", "0=2", "--bump-extra-core", "1=3",
            "--bump-build", "0=4", "--bump-build", "1=5",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.1.dev.3+2")]  // Secondary bumps preserve schema structure
    fn test_secondary_schema_preserve_structure(
        #[case] expected: &str,
    ) {
        // Create fixture with build component
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_build(&[1]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-dev", "--bump-build", "0", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}

mod all_category_combinations {
    use super::*;

    #[rstest]
    #[case("3.4.5-beta.2.1.1+1")]  // primary + secondary + schema
    fn test_all_categories_simple(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0])
            .with_build(&[0]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-label", "beta", "--bump-pre-release-num",
            "--bump-extra-core", "0", "--bump-build", "0",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("5.7.9-rc.3.2.3+4.5")]  // Complex combination with custom values
    fn test_all_categories_complex(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core and build components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0, 0])
            .with_build(&[0, 0]);

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "4", "--bump-minor", "5", "--bump-patch", "6",
            "--bump-pre-release-label", "rc", "--bump-pre-release-num", "2",
            "--bump-epoch", "3",
            "--bump-extra-core", "0=2", "--bump-extra-core", "1=3",
            "--bump-build", "0=4", "--bump-build", "1=5",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.4.5-beta.2.1.1+1+d5.dirty")]  // All categories with VCS context
    fn test_all_categories_with_context(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core, build, and VCS components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0])
            .with_build(&[0])

            .with_distance(
                Some("v1.2.3-alpha.1".to_string()),
                Some(5),
                Some(true),
            );

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-label", "beta", "--bump-pre-release-num",
            "--bump-extra-core", "0", "--bump-build", "0",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("3.4.5-beta.2.1.1+1")]  // All categories without VCS context
    fn test_all_categories_without_context(
        #[case] expected: &str,
    ) {
        // Create fixture with extra-core, build, and VCS components
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()
            .with_extra_core(&[0])
            .with_build(&[0])

            .with_distance(
                Some("v1.2.3-alpha.1".to_string()),
                Some(5),
                Some(true),
            );

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-label", "beta", "--bump-pre-release-num",
            "--bump-extra-core", "0", "--bump-build", "0",
            "--no-bump-context", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("6.8.10-rc.4.3.4+5.6.dev.7.post.8")]  // Maximum complexity scenario
    fn test_maximum_complexity_combination(
        full_combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "5", "--bump-minor", "6", "--bump-patch", "7",
            "--bump-pre-release-label", "rc", "--bump-pre-release-num", "3",
            "--bump-epoch", "4", "--bump-post", "6", "--bump-dev", "7",
            "--bump-core", "0=5", "--bump-core", "1=6", "--bump-core", "2=7",
            "--bump-extra-core", "0=3", "--bump-extra-core", "1=3",
            "--bump-build", "0=3",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}

mod context_with_combinations {
    use super::*;

    #[rstest]
    #[case("3.4.5-beta.2+d5.dirty", "3.4.5-beta.2")]  // With vs without context
    fn test_context_impact_on_combinations(
        #[case] with_context: &str,
        #[case] without_context: &str,
    ) {
        // Create fixture with VCS data
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()

            .with_distance(
                Some("v1.2.3-alpha.1".to_string()),
                Some(5),
                Some(true),
            );

        let input = fixture.to_ron();

        // Test with context (default)
        let output_with = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-label", "beta", "--bump-pre-release-num",
            "--output-format", ZERV
        ], &input);

        // Test without context
        let output_without = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--bump-minor", "--bump-patch",
            "--bump-pre-release-label", "beta", "--bump-pre-release-num",
            "--no-bump-context", "--output-format", ZERV
        ], &input);

        assert_eq!(output_with.trim(), with_context);
        assert_eq!(output_without.trim(), without_context);
    }

    #[rstest]
    #[case("2!2.3.4-alpha.2+d5.dirty")]  // Context with complex combination
    fn test_context_with_schema_bumps(
        full_combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-epoch", "--bump-major", "--bump-patch",
            "--bump-pre-release-num", "--bump-core", "1",
            "--bump-extra-core", "0", "--bump-build", "0",
            "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2!2.3.4-alpha.2")]  // No context with complex combination
    fn test_no_context_with_schema_bumps(
        full_combination_fixture: ZervFixture,
        #[case] expected: &str,
    ) {
        let input = full_combination_fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-epoch", "--bump-major", "--bump-patch",
            "--bump-pre-release-num", "--bump-core", "1",
            "--bump-extra-core", "0", "--bump-build", "0",
            "--no-bump-context", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case("2.0.0-alpha.1")]  // Context doesn't affect pure tag versions
    fn test_context_with_pure_tag_version(
        #[case] expected: &str,
    ) {
        // Create pure tag version (no VCS data)
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1))
            .with_standard_tier_1()

            .with_distance(
                Some("v1.2.3-alpha.1".to_string()),
                None,   // no distance
                None,   // no dirty state
            );

        let input = fixture.to_ron();
        let output = TestCommand::run_with_stdin(&[
            "version", "--source", STDIN,
            "--bump-major", "--output-format", ZERV
        ], &input);

        assert_eq!(output.trim(), expected);
    }
}
