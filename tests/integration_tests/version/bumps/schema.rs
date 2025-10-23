//! Schema component bump tests
//!
//! Tests for --bump-core, --bump-extra-core, and --bump-build options.
//! These tests verify that schema-based component bumps work correctly
//! with index=value syntax and proper value resolution.

use rstest::{fixture, rstest};
use crate::util::TestCommand;
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::{
    sources::STDIN,
    formats::SEMVER,
    formats::ZERV,
    schema_names::ZERV_STANDARD,
    schema_names::ZERV_CALVER,
};
use zerv::version::PreReleaseLabel;

/// Zerv fixture with standard schema for bump tests
#[fixture]
fn standard_schema_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_standard_tier_1()
}

/// Zerv fixture with calver schema for bump tests
#[fixture]
fn calver_schema_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(2023, 12, 25)
        .with_pre_release(PreReleaseLabel::Beta, Some(2))
        .with_schema(ZERV_CALVER)
}

/// Zerv fixture with VCS components for build bump tests
#[fixture]
fn schema_with_vcs_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_pre_release(PreReleaseLabel::Alpha, Some(1))
        .with_distance(5)
        .with_standard_tier_2()
}

mod core_bump {
    use super::*;

    #[rstest]
    #[case::major("5.2.3-alpha.1", "0=5")]  // Bump major (index 0) in standard schema
    #[case::minor("1.7.3-alpha.1", "1=7")]  // Bump minor (index 1) in standard schema
    #[case::patch("1.2.9-alpha.1", "2=9")]  // Bump patch (index 2) in standard schema
    fn test_bump_core_standard_schema(
        standard_schema_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_arg: &str,
    ) {
        let input = standard_schema_fixture.build().to_string();
        let args = format!("version --source {} --bump-core {} --output-format {}", STDIN, bump_arg, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[rstest]
    #[case::year("2025.12.25-beta.2", "0=2025")]  // Bump year in calver schema
    #[case::month("2023.15.25-beta.2", "1=15")]  // Bump month in calver schema
    #[case::day("2023.12.30-beta.2", "2=30")]  // Bump day in calver schema
    fn test_bump_core_calver_schema(
        calver_schema_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_arg: &str,
    ) {
        let input = calver_schema_fixture.build().to_string();
        let args = format!("version --source {} --bump-core {} --output-format {}", STDIN, bump_arg, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[test]
    fn test_bump_core_with_index_out_of_bounds_errors() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!("version --source {} --bump-core 10=5 --output-format {}", STDIN, ZERV);

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
    }
}

mod extra_core_bump {
    use super::*;

    #[rstest]
    #[case::epoch("2!1.2.3-alpha.1", "0=2")]  // Bump epoch in standard schema
    #[case::post("1.2.3-alpha.1.post.5", "2=5")]  // Bump post in standard schema
    fn test_bump_extra_core_standard_schema(
        standard_schema_fixture: ZervFixture,
        #[case] expected: &str,
        #[case] bump_arg: &str,
    ) {
        let input = standard_schema_fixture.build().to_string();
        let args = format!("version --source {} --bump-extra-core {} --output-format {}", STDIN, bump_arg, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), expected);
    }

    #[test]
    fn test_bump_extra_core_with_empty_components() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1();
        let input = fixture.build().to_string();
        let args = format!("version --source {} --bump-extra-core 0=2 --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        // Should work even with initially empty components
        assert_eq!(output.trim(), "2!1.2.3");
    }

    #[test]
    fn test_bump_extra_core_with_multiple_values() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!("version --source {} --bump-extra-core 0=3 --bump-extra-core 2=7 --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), "3!1.2.3-alpha.1.post.7");
    }
}

mod build_bump {
    use super::*;

    #[rstest]
    #[case::valid_index("test-branch", "0=test-branch")]  // Valid build index in tier 2
    fn test_bump_build_with_valid_index(
        schema_with_vcs_fixture: ZervFixture,
        #[case] expected_substring: &str,
        #[case] bump_arg: &str,
    ) {
        let input = schema_with_vcs_fixture.build().to_string();
        let args = format!("version --source {} --bump-build {} --output-format {}", STDIN, bump_arg, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert!(output.contains(expected_substring));
    }

    #[test]
    fn test_bump_build_with_vcs_derived_component_fails() {
        let input = schema_with_vcs_fixture().build().to_string();
        let args = format!("version --source {} --bump-build 0=test --output-format {}", STDIN, ZERV);

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
    }

    #[test]
    fn test_bump_build_with_empty_build_section_fails() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!("version --source {} --bump-build 0=test --output-format {}", STDIN, ZERV);

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
    }
}

mod schema_combinations {
    use super::*;

    #[test]
    fn test_multiple_schema_bumps_combined() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 0=5 --bump-extra-core 0=2 --bump-extra-core 2=8 --output-format {}",
            STDIN, ZERV
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), "2!5.2.3-alpha.1.post.8");
    }

    #[test]
    fn test_schema_bumps_preserve_order() {
        let input = calver_schema_fixture().build().to_string();
        let args = format!(
            "version --source {} --bump-core 0=2024 --bump-core 2=28 --bump-extra-core 0=1 --output-format {}",
            STDIN, ZERV
        );
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), "1!2024.12.28-beta.2");
    }

    #[test]
    fn test_schema_bumps_with_different_schemas() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Rc, Some(1))
            .with_schema(ZERV_STANDARD);
        let input = fixture.build().to_string();
        let args = format!("version --source {} --bump-core 1=8 --output-format {}", STDIN, ZERV);
        let output = TestCommand::run_with_stdin(&args, input);

        assert_eq!(output.trim(), "1.8.3-rc.1");
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

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
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

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
    }

    #[test]
    fn test_schema_bump_non_numeric_value_fails() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!("version --source {} --bump-core 0=not_a_number --output-format {}", STDIN, ZERV);

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
    }

    #[test]
    fn test_schema_bump_too_large_index_fails() {
        let input = standard_schema_fixture().build().to_string();
        let args = format!("version --source {} --bump-core 10=5 --output-format {}", STDIN, ZERV);

        TestCommand::new()
            .args_from_str(&args)
            .stdin(input)
            .assert_failure();
    }
}
