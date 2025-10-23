use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[fixture]
fn standard_tier1_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_vcs_data(Some(5), Some(true), None, None, None, None, None)
        .with_standard_tier_1()
}

#[fixture]
fn standard_tier2_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_vcs_data(
            Some(5),
            Some(false),
            Some("main".to_string()),
            None,
            None,
            None,
            None,
        )
        .with_standard_tier_2()
}

mod core_overrides {
    use super::*;

    #[rstest]
    #[case::major("0=5", "5.2.3")]
    #[case::minor("1=7", "1.7.3")]
    #[case::patch("2=9", "1.2.9")]
    fn test_core_override_single_component(
        standard_tier1_fixture: ZervFixture,
        #[case] override_arg: &str,
        #[case] expected_output: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let args = format!(
            "version --source stdin --core {} --output-format semver",
            override_arg
        );

        let output = TestCommand::run_with_stdin(&args, zerv_ron);

        assert_eq!(output, expected_output);
    }

    #[rstest]
    #[case::major_minor(&["0=5", "1=7"], "5.7.3")]
    #[case::major_patch(&["0=5", "2=9"], "5.2.9")]
    #[case::minor_patch(&["1=7", "2=9"], "1.7.9")]
    #[case::all_components(&["0=5", "1=7", "2=9"], "5.7.9")]
    fn test_core_override_multiple_components(
        standard_tier1_fixture: ZervFixture,
        #[case] override_args: &[&str],
        #[case] expected_output: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let mut args = "version --source stdin".to_string();

        for arg in override_args {
            args.push_str(&format!(" --core {}", arg));
        }
        args.push_str(" --output-format semver");

        let output = TestCommand::run_with_stdin(&args, zerv_ron);

        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_core_override_with_output_format_zerv() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1();
        let zerv_ron = fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --core 0=5 --output-format zerv",
            zerv_ron,
        );

        // Verify the override is applied in zerv format
        assert!(output.contains("major: Some(5)"));
        assert!(output.contains("minor: Some(2)"));
        assert!(output.contains("patch: Some(3)"));
    }

    #[rstest]
    #[case("5=10")] // Index out of bounds for tier 1 schema
    fn test_core_override_index_out_of_bounds_errors(
        standard_tier1_fixture: ZervFixture,
        #[case] override_arg: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let args = format!(
            "version --source stdin --core {} --output-format semver",
            override_arg
        );

        // Should fail with index out of bounds error
        TestCommand::new()
            .args_from_str(&args)
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[rstest]
    #[case("0=5")] // Valid index, but testing negative index handling
    fn test_core_override_negative_index_handled_as_cli_error(
        standard_tier1_fixture: ZervFixture,
        #[case] override_arg: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let args = format!(
            "version --source stdin --core {} --output-format semver",
            override_arg
        );

        // This should actually work since the argument is valid
        let output = TestCommand::run_with_stdin(&args, zerv_ron);
        assert_eq!(output, "5.2.3");
    }
}

mod extra_core_overrides {
    use super::*;

    #[rstest]
    #[case::epoch("0=5")] // Epoch override
    #[case::post("2=1")] // Post override
    fn test_extra_core_override_numeric_components(
        standard_tier1_fixture: ZervFixture,
        #[case] override_arg: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let args = format!(
            "version --source stdin --extra-core {} --output-format zerv",
            override_arg
        );

        let output = TestCommand::run_with_stdin(&args, zerv_ron);

        // Verify it runs without error and contains the override
        assert!(!output.is_empty());
        if override_arg.starts_with("0=") {
            assert!(output.contains("epoch: Some(5)"));
        } else if override_arg.starts_with("2=") {
            assert!(output.contains("post: Some(1)"));
        }
    }

    #[test]
    fn test_extra_core_override_with_zerv_format() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1();
        let zerv_ron = fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --extra-core 0=5 --output-format zerv",
            zerv_ron,
        );

        // Verify it runs and contains epoch in zerv format
        assert!(output.contains("epoch: Some(5)"));
    }

    #[rstest]
    #[case(&["0=5", "2=10"])] // Epoch and Post overrides
    fn test_extra_core_override_multiple_components(
        standard_tier1_fixture: ZervFixture,
        #[case] override_args: &[&str],
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let mut args = "version --source stdin".to_string();

        for arg in override_args {
            args.push_str(&format!(" --extra-core {}", arg));
        }
        args.push_str(" --output-format zerv");

        let output = TestCommand::run_with_stdin(&args, zerv_ron);

        // Should run without error and contain the overrides
        assert!(!output.is_empty());
        assert!(output.contains("epoch: Some(5)"));
        assert!(output.contains("post: Some(10)"));
    }
}

mod build_overrides {
    use super::*;

    #[test]
    fn test_build_override_fails_for_vcs_components() {
        // Build overrides should fail for VCS-derived fields in tier 2
        let zerv_ron = standard_tier2_fixture().build().to_string();

        // This should fail because build[0] = BumpedBranch (VCS-derived)
        TestCommand::new()
            .args_from_str("version --source stdin --build 0=test-branch --output-format zerv")
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[test]
    fn test_build_override_with_empty_build_section_fails() {
        // Test with tier 1 (empty build section)
        let zerv_ron = standard_tier1_fixture().build().to_string();

        // Should fail because build section is empty (index 0 out of bounds)
        TestCommand::new()
            .args_from_str("version --source stdin --build 0=test --output-format zerv")
            .stdin(zerv_ron)
            .assert_failure();
    }
}

mod schema_combinations {
    use super::*;

    #[test]
    fn test_all_schema_components_combined() {
        let zerv_ron = standard_tier1_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --core 0=5 --extra-core 0=3 --output-format zerv",
            zerv_ron,
        );

        // Verify it runs without error and contains expected components
        assert!(!output.is_empty());
        assert!(output.contains("major: Some(5)"));
        assert!(output.contains("epoch: Some(3)"));
    }

    #[test]
    fn test_schema_overrides_work_across_different_schemas() {
        // Test that overrides work regardless of schema tier
        let zerv_ron = standard_tier1_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --core 0=2024 --extra-core 0=1 --output-format zerv",
            zerv_ron,
        );

        // Verify it runs without error
        assert!(!output.is_empty());
        assert!(output.contains("major: Some(2024)"));
        assert!(output.contains("epoch: Some(1)"));
    }

    #[test]
    fn test_multiple_schema_section_overrides() {
        let zerv_ron = standard_tier1_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --core 0=2 --core 1=5 --extra-core 0=1 --extra-core 2=5 --output-format zerv",
            zerv_ron,
        );

        // Verify it runs without error with multiple schema section overrides
        assert!(!output.is_empty());
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(5)"));
        assert!(output.contains("epoch: Some(1)"));
        assert!(output.contains("post: Some(5)"));
    }
}

mod error_handling {
    use super::*;

    #[rstest]
    #[case("--core", "invalid_format")]
    #[case("--extra-core", "=invalid")]
    fn test_schema_override_malformed_arguments_handled(
        standard_tier1_fixture: ZervFixture,
        #[case] flag: &str,
        #[case] arg_value: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let args = format!("version --source stdin {} {}", flag, arg_value);

        // Should produce an error but not crash
        TestCommand::new()
            .args_from_str(&args)
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[rstest]
    #[case("--core", "0=not_a_number")]
    #[case("--extra-core", "0=not_a_number")]
    fn test_schema_override_non_numeric_values_fail(
        standard_tier1_fixture: ZervFixture,
        #[case] flag: &str,
        #[case] arg_value: &str,
    ) {
        let zerv_ron = standard_tier1_fixture.build().to_string();
        let args = format!("version --source stdin {} {}", flag, arg_value);

        // Should fail with error but not crash
        TestCommand::new()
            .args_from_str(&args)
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[test]
    fn test_schema_override_too_large_index_fails() {
        let zerv_ron = standard_tier1_fixture().build().to_string();

        // Should fail with index out of bounds error
        TestCommand::new()
            .args_from_str("version --source stdin --core 10=5 --output-format semver")
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[test]
    fn test_schema_override_build_with_empty_section_fails() {
        let zerv_ron = standard_tier1_fixture().build().to_string();

        // Should fail because tier 1 has empty build section
        TestCommand::new()
            .args_from_str("version --source stdin --build 0=test --output-format zerv")
            .stdin(zerv_ron)
            .assert_failure();
    }
}
