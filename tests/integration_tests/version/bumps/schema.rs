//! Schema component bump tests
//!
//! Tests for --bump-core, --bump-extra-core, and --bump-build options.
//! These tests verify that schema-based component bumps work correctly
//! with index=value syntax and proper value resolution.

use rstest::{
    fixture,
    rstest,
};
use zerv::schema::ZervSchemaPreset;
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::formats::SEMVER;
use zerv::utils::constants::sources::STDIN;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

/// Zerv fixture with standard schema for bump tests
#[fixture]
fn standard_schema_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePost)
}

/// Zerv fixture with calver schema for bump tests
#[fixture]
fn calver_schema_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(2023, 12, 25)
        .with_pre_release(PreReleaseLabel::Beta, Some(2))
        .with_schema_preset(ZervSchemaPreset::CalverBasePrereleasePost)
}

/// Zerv fixture with VCS components for build bump tests
#[fixture]
fn schema_with_vcs_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(5)
        .with_schema_preset(ZervSchemaPreset::StandardBasePrereleasePostContext)
}

mod core_bump {
    use super::*;

    #[rstest]
    #[case::major("2.0.0", "0=1")] // Bump major by 1 (1 + 1 = 2), resets minor, patch, prerelease
    #[case::major_by_5("6.0.0", "0=5")] // Bump major by 5 (1 + 5 = 6), resets lower components
    #[case::minor("1.3.0", "1=1")] // Bump minor by 1 (2 + 1 = 3), resets patch, prerelease
    #[case::minor_by_7("1.9.0", "1=7")] // Bump minor by 7 (2 + 7 = 9), resets patch, prerelease
    #[case::patch("1.2.4", "2=1")] // Bump patch by 1 (3 + 1 = 4), resets prerelease
    #[case::patch_by_6("1.2.9", "2=6")] // Bump patch by 6 (3 + 6 = 9), resets prerelease
    fn test_bump_core_standard_schema(
        standard_schema_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_arg: &str,
    ) {
        let input = standard_schema_fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-core {} --output-format {}",
            STDIN, bump_arg, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[test]
    fn test_bump_core_calver_with_timestamp_fails() {
        let input = calver_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 0=1 --output-format {}",
            STDIN, SEMVER
        );

        TestCommand::run_with_stdin_expect_fail(&args, input);
    }

    #[test]
    fn test_bump_core_with_index_out_of_bounds_errors() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 10=5 --output-format {}",
            STDIN, SEMVER
        );

        TestCommand::run_with_stdin_expect_fail(&args, input);
    }
}

mod extra_core_bump {
    use super::*;

    #[test]
    fn test_bump_extra_core_epoch() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-extra-core 0=2 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump epoch by 2 (0 + 2 = 2), resets all lower components (major, minor, patch, pre-release)
        // SemVer formats epoch as prerelease component
        assert_eq!(output.trim(), "0.0.0-epoch.2");
    }

    #[test]
    fn test_bump_extra_core_post() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-extra-core 2=5 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump post by 5 (0 + 5 = 5), post doesn't reset pre-release in semver
        assert_eq!(output.trim(), "1.2.3-alpha.1.post.5");
    }

    #[test]
    fn test_bump_extra_core_with_empty_epoch() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease);
        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-extra-core 0=2 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump epoch from None (0) by 2 = 2, resets major, minor, patch
        // SemVer formats epoch as prerelease component
        assert_eq!(output.trim(), "0.0.0-epoch.2");
    }

    #[test]
    fn test_bump_extra_core_with_multiple_values() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-extra-core 0=3 --bump-extra-core 2=7 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump epoch by 3 (0 + 3 = 3), resets all lower, then bump post by 7 (0 + 7 = 7)
        // SemVer formats both epoch and post as prerelease components
        assert_eq!(output.trim(), "0.0.0-epoch.3.post.7");
    }
}

mod build_bump {
    use super::*;

    #[test]
    fn test_bump_build_string_component() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_build(zerv::version::zerv::components::Component::Str(
                "alpha".to_string(),
            ));
        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-build 0=beta --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // String components: bump replaces the value
        assert!(output.contains("+beta"));
    }

    #[test]
    fn test_bump_build_uint_component() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .with_build(zerv::version::zerv::components::Component::UInt(10));
        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-build 0=5 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // UInt components: bump increments (10 + 5 = 15)
        assert!(output.contains("+15"));
    }

    #[test]
    fn test_bump_build_with_vcs_derived_component_fails() {
        let input = schema_with_vcs_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-build 0=test --output-format {}",
            STDIN, SEMVER
        );

        // schema_with_vcs_fixture has VCS-derived build components which cannot be bumped
        TestCommand::run_with_stdin_expect_fail(&args, input);
    }

    #[test]
    fn test_bump_build_with_empty_build_section_fails() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-build 0=test --output-format {}",
            STDIN, SEMVER
        );

        // standard_schema_fixture has no build components
        TestCommand::run_with_stdin_expect_fail(&args, input);
    }
}

mod schema_combinations {
    use super::*;

    #[test]
    fn test_multiple_schema_bumps_combined() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-extra-core 0=2 --bump-extra-core 2=8 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump epoch by 2 (0 + 2 = 2) resets all lower, then bump post by 8 (0 + 8 = 8)
        // SemVer formats both epoch and post as prerelease components
        assert_eq!(output.trim(), "0.0.0-epoch.2.post.8");
    }

    #[test]
    fn test_schema_bumps_with_minor_only() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease);
        let input = fixture.build().to_string();
        let args = format!(
            "version --source {} --bump-core 1=8 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump minor by 8 (2 + 8 = 10), resets patch and prerelease
        assert_eq!(output.trim(), "1.10.0");
    }

    #[test]
    fn test_schema_bumps_with_multiple_core_components() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 0=1 --bump-core 2=5 --output-format {}",
            STDIN, SEMVER
        );
        let output = TestCommand::run_with_stdin(&args, input);

        // Bump major by 1 (1 + 1 = 2) resets minor and patch, then bump patch by 5 (0 + 5 = 5)
        assert_eq!(output.trim(), "2.0.5");
    }
}

mod error_handling {
    use super::*;

    #[rstest]
    #[case::invalid_format("invalid_format", "--bump-core")]
    #[case::missing_equals("=5", "--bump-core")]
    #[case::missing_value("0=", "--bump-extra-core")]
    fn test_schema_bump_malformed_arguments(
        standard_schema_fixture: ZervFixture,
        #[case] arg_value: &str,
        #[case] flag: &str,
    ) {
        let input = standard_schema_fixture.build().to_string();
        let args = format!("version --source {} {} {}", STDIN, flag, arg_value);

        TestCommand::run_with_stdin_expect_fail(&args, input);
    }

    #[rstest]
    #[case::negative_index("-1=5", "--bump-core")]
    #[case::negative_index("-2=3", "--bump-extra-core")]
    fn test_schema_bump_negative_index_handled_as_cli_error(
        standard_schema_fixture: ZervFixture,
        #[case] arg_value: &str,
        #[case] flag: &str,
    ) {
        let input = standard_schema_fixture.build().to_string();
        let args = format!("version --source {} {} {}", STDIN, flag, arg_value);

        TestCommand::run_with_stdin_expect_fail(&args, input);
    }

    #[test]
    fn test_schema_bump_non_numeric_value_fails() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 0=not_a_number --output-format {}",
            STDIN, SEMVER
        );

        TestCommand::run_with_stdin_expect_fail(&args, input);
    }

    #[test]
    fn test_schema_bump_too_large_index_fails() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 10=5 --output-format {}",
            STDIN, SEMVER
        );

        TestCommand::run_with_stdin_expect_fail(&args, input);
    }
}
