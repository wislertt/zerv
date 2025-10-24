//! MainConfig + OverrideConfig interaction tests
//!
//! Tests the interaction between MainConfig options (source, format, schema, template, directory)
//! and OverrideConfig options (VCS, components, schema, custom variables).

use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[fixture]
fn base_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(2, 1, 0)
        .with_vcs_data(
            Some(5),
            Some(true),
            Some("feature/test-branch".to_string()),
            Some("abc123def456".to_string()),
            None,
            None,
            None,
        )
        .with_standard_tier_1()
}

#[fixture]
fn calver_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(2023, 10, 25)
        .with_vcs_data(
            Some(3),
            Some(false),
            Some("release".to_string()),
            Some("xyz789abc123".to_string()),
            None,
            None,
            None,
        )
        .with_calver_tier_1()
}

mod stdin_override_combinations {
    use super::*;

    #[rstest]
    #[case::stdin_with_tag_version("--tag-version", "2.1.0", "2.1.0")]
    #[case::stdin_with_major_override("--major", "5", "5.1.0")]
    #[case::stdin_with_minor_override("--minor", "3", "2.3.0")]
    #[case::stdin_with_patch_override("--patch", "7", "2.1.7")]
    fn test_stdin_source_with_basic_overrides(
        base_fixture: ZervFixture,
        #[case] override_arg: &str,
        #[case] value: &str,
        #[case] expected_prefix: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin {} {} --output-format semver",
                override_arg, value
            ),
            zerv_ron,
        );
        assert_eq!(output, expected_prefix);
    }

    #[rstest]
    #[case::stdin_with_distance_override("--distance", "10", "2.1.0")]
    fn test_stdin_source_with_vcs_overrides(
        base_fixture: ZervFixture,
        #[case] override_arg: &str,
        #[case] value: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &match override_arg {
                "--distance" => format!(
                    "version --source stdin {} {} --output-format semver",
                    override_arg, value
                ),
                "--dirty" => format!(
                    "version --source stdin {} --output-format semver",
                    override_arg
                ),
                _ => unreachable!(),
            },
            zerv_ron,
        );
        assert_eq!(output, expected); // Semver format ignores VCS data
    }

    #[test]
    fn test_stdin_source_with_dirty_override() {
        let zerv_ron = base_fixture().build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --dirty --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "2.1.0"); // Semver format ignores VCS data
    }

    #[rstest]
    fn test_stdin_source_with_multiple_overrides(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 3 --minor 5 --distance 8 --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "3.5.0"); // Semver format ignores VCS data like distance
    }
}

mod format_override_combinations {
    use super::*;

    #[rstest]
    #[case::semver_input_with_major_override(
        "--input-format semver",
        "--major",
        "5",
        "--output-format semver",
        "5.1.0"
    )]
    #[case::semver_input_with_minor_override(
        "--input-format semver",
        "--minor",
        "8",
        "--output-format semver",
        "2.8.0"
    )]
    fn test_input_format_with_component_overrides(
        base_fixture: ZervFixture,
        #[case] input_format: &str,
        #[case] override_arg: &str,
        #[case] value: &str,
        #[case] output_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin {} {} {} {} {}",
                input_format, override_arg, value, output_format, "--distance 5"
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case::semver_to_pep440_with_overrides(
        "--input-format semver",
        "--output-format pep440",
        "--major 3 --minor 7"
    )]
    #[case::semver_to_zerv_with_overrides(
        "--input-format semver",
        "--output-format zerv",
        "--major 4"
    )]
    fn test_format_conversion_with_overrides(
        base_fixture: ZervFixture,
        #[case] input_format: &str,
        #[case] output_format: &str,
        #[case] overrides: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin {} {} {}",
                input_format, output_format, overrides
            ),
            zerv_ron,
        );

        // Verify output contains the overridden major version
        if overrides.contains("--major 3") {
            assert!(output.starts_with("3.7.0"));
        } else if overrides.contains("--major 4") {
            // Zerv format outputs RON structure, check for major: Some(4)
            assert!(output.contains("major: Some(4)"));
        }
        assert!(!output.is_empty());
    }
}

mod schema_override_combinations {
    use super::*;

    #[rstest]
    #[case::standard_schema_with_major_override(
        "--schema zerv-standard",
        "--major",
        "5",
        "5.1.0+feature.test.branch.5.abc123d"
    )]
    fn test_schema_preset_with_component_overrides(
        base_fixture: ZervFixture,
        #[case] schema_arg: &str,
        #[case] override_arg: &str,
        #[case] value: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin {} {} {} --output-format semver",
                schema_arg, override_arg, value
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_calver_schema_with_year_override_dynamic() {
        let base_fixture = ZervFixture::new()
            .with_version(2, 1, 0)
            .with_vcs_data(
                Some(5),
                Some(true),
                Some("feature/test-branch".to_string()),
                Some("abc123def456".to_string()),
                None,
                None,
                None,
            )
            .with_standard_tier_1();

        let zerv_ron = base_fixture.build().to_string();
        let today_date = chrono::Utc::now().format("%Y.%m.%d").to_string();
        let expected = format!("{}-5+feature.test.branch.5.abc123d", today_date);

        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-calver --patch 5 --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_schema_component_override_interaction(base_fixture: ZervFixture) {
        // Test interaction between --schema and --core/--extra-core overrides
        let zerv_ron = base_fixture.build().to_string();

        // This should override core component 0 (major) to 3
        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema zerv-standard --core 0=3 --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "3.1.0+feature.test.branch.5.abc123d");
    }
}

mod template_override_combinations {
    use super::*;

    #[rstest]
    #[case::basic_template_with_major_override(
        "v{{major}}.{{minor}}.{{patch}}",
        "--major 5",
        "v5.1.0"
    )]
    #[case::basic_template_with_minor_override(
        "{{major}}.{{minor}}.{{patch}}",
        "--minor 3",
        "2.3.0"
    )]
    #[case::template_with_vcs_distance(
        "{{major}}.{{minor}}.{{patch}}+{{distance}}",
        "--distance 12",
        "2.1.0+12"
    )]
    fn test_template_rendering_with_overrides(
        base_fixture: ZervFixture,
        #[case] template: &str,
        #[case] overrides: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = base_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-template '{}' {}",
                template, overrides
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_template_with_sanitize_helper(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        // Test sanitize helper with overridden values
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch 'feature/test' --output-template '{{sanitize bumped_branch preset=\"dotted\"}}'",
            zerv_ron,
        );
        assert_eq!(output, "feature.test");
    }

    #[rstest]
    fn test_template_with_hash_helper(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        // Test hash helper with overridden branch name
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bumped-branch 'feature/very-long-branch-name-for-testing' --output-template '{{hash bumped_branch}}'",
            zerv_ron,
        );

        // Should be a 7-character hash
        assert_eq!(output.len(), 7);
        assert!(output.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[rstest]
    fn test_template_with_custom_variables(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        // Test template with custom variable overrides
        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --custom '{"build_type": "release", "env": "prod"}' --output-template '{{major}}.{{minor}}.{{patch}}-{{custom.build_type}}-{{custom.env}}'"#,
            zerv_ron,
        );
        assert_eq!(output, "2.1.0-release-prod");
    }

    #[rstest]
    fn test_complex_template_with_multiple_overrides(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();
        let template = "build-{{major}}.{{minor}}.{{patch}}+{{distance}}-{{bumped_branch}}";

        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --major 3 --distance 8 --bumped-branch 'release' --output-template '{}'",
                template
            ),
            zerv_ron,
        );
        assert_eq!(output, "build-3.1.0+8-release");
    }
}

mod error_scenarios {
    use super::*;

    #[test]
    fn test_conflicting_overrides_with_main_config() {
        let zerv_ron = base_fixture().build().to_string();

        // Test conflicting dirty flags (--dirty vs --clean)
        let args = "version --source stdin --dirty --clean";
        TestCommand::new()
            .args_from_str(args)
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[test]
    fn test_invalid_schema_override_combination() {
        let zerv_ron = base_fixture().build().to_string();

        // Test invalid schema with overrides
        let args = "version --source stdin --schema invalid-schema --major 5";
        TestCommand::new()
            .args_from_str(args)
            .stdin(zerv_ron)
            .assert_failure();
    }

    #[test]
    fn test_template_error_with_invalid_override() {
        let zerv_ron = base_fixture().build().to_string();

        // Test template with invalid custom variable reference - this succeeds but outputs empty string
        let output = TestCommand::run_with_stdin(
            "version --source stdin --custom '{}' --output-template '{{custom.nonexistent}}'",
            zerv_ron,
        );
        assert_eq!(output, ""); // Missing custom variables render as empty strings
    }

    #[test]
    fn test_invalid_core_override() {
        let zerv_ron = base_fixture().build().to_string();

        // Test invalid core component index
        let args = "version --source stdin --core 10=5 --output-format semver";
        TestCommand::new()
            .args_from_str(args)
            .stdin(zerv_ron)
            .assert_failure();
    }
}
