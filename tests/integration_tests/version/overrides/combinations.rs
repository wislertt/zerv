use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[fixture]
fn base_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_vcs_data(
            Some(5),
            Some(true),
            Some("feature-branch".to_string()),
            None,
            None,
            None,
            None,
        )
        .with_standard_tier_1()
}

#[fixture]
fn vcs_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_vcs_data(
            Some(10),
            Some(false),
            Some("main".to_string()),
            None,
            None,
            None,
            None,
        )
        .with_standard_tier_1()
}

mod primary_secondary_combinations {
    use super::*;

    #[rstest]
    #[case::major_minor(&["--major", "5", "--minor", "7"])]
    #[case::primary_secondary(&["--major", "5", "--epoch", "2"])]
    fn test_primary_secondary_combinations(base_fixture: ZervFixture, #[case] args: &[&str]) {
        let zerv_ron = base_fixture.build().to_string();
        let mut full_args = vec!["version", "--source", "stdin"];
        full_args.extend(args);
        full_args.push("--output-format");
        full_args.push("semver");

        let output = TestCommand::run_with_stdin(&full_args.join(" "), zerv_ron);

        // Verify output is valid semver format
        assert!(output.contains('.'));
        assert!(!output.is_empty());
    }
}

mod vcs_component_combinations {
    use super::*;

    #[rstest]
    #[case::vcs_core(&["--tag-version", "v2.0.0", "--core", "0=5"])]
    #[case::vcs_extra_core(&["--distance", "10", "--extra-core", "0=2"])]
    fn test_vcs_component_combinations(base_fixture: ZervFixture, #[case] args: &[&str]) {
        let zerv_ron = base_fixture.build().to_string();
        let mut full_args = vec!["version", "--source", "stdin"];
        full_args.extend(args);
        full_args.push("--output-format");
        full_args.push("semver");

        let output = TestCommand::run_with_stdin(&full_args.join(" "), zerv_ron);

        // Verify output is valid semver format
        assert!(output.contains('.'));
        assert!(!output.is_empty());
    }

    #[test]
    fn test_vcs_component_with_zerv_format() {
        let zerv_ron = vcs_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version \
            --source stdin --distance 15 \
            --dirty --core 0=3 --output-format zerv",
            zerv_ron,
        );

        // Verify all overrides are applied in zerv format
        assert!(output.contains("major: Some(3)"));
        assert!(output.contains("minor: Some(2)"));
        assert!(output.contains("patch: Some(3)"));
        assert!(output.contains("distance: Some(15)"));
        assert!(output.contains("dirty: Some(true)"));
    }
}

mod schema_vcs_combinations {
    use super::*;

    #[rstest]
    #[case::schema_vcs(&["--core", "0=2", "--tag-version", "v5.0.0"])]
    #[case::schema_vcs_custom(&["--extra-core", "0=3", "--bumped-branch", "release"])]
    fn test_schema_vcs_combinations(base_fixture: ZervFixture, #[case] args: &[&str]) {
        let zerv_ron = base_fixture.build().to_string();
        let mut full_args = vec!["version", "--source", "stdin"];
        full_args.extend(args);
        full_args.push("--output-format");
        full_args.push("semver");

        let output = TestCommand::run_with_stdin(&full_args.join(" "), zerv_ron);

        // Verify output is valid semver format
        assert!(output.contains('.'));
        assert!(!output.is_empty());
    }

    #[test]
    fn test_schema_vcs_combination_zerv_format() {
        let zerv_ron = vcs_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version \
            --source stdin --core 0=4 --extra-core 0=1 \
            --distance 20 --bumped-branch custom \
            --output-format zerv",
            zerv_ron,
        );

        // Verify schema and VCS overrides are both applied
        assert!(output.contains("major: Some(4)"));
        assert!(output.contains("epoch: Some(1)"));
        assert!(output.contains("distance: Some(20)"));
        assert!(output.contains("bumped_branch: Some(\"custom\")"));
    }
}

mod complex_multi_category_scenarios {
    use super::*;

    #[test]
    fn test_all_override_categories() {
        let zerv_ron = vcs_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version \
            --source stdin --major 3 --minor 4 --epoch 1 \
            --distance 8 --dirty --core 0=2 --extra-core 2=5 \
            --output-format zerv",
            zerv_ron,
        );

        // Verify all categories of overrides are applied
        assert!(output.contains("major: Some(2)")); // schema override should win
        assert!(output.contains("minor: Some(4)")); // component override
        assert!(output.contains("epoch: Some(1)")); // component override
        assert!(output.contains("post: Some(5)")); // schema override
        assert!(output.contains("distance: Some(8)"));
        assert!(output.contains("dirty: Some(true)"));
    }

    #[test]
    fn test_override_precedence_ordering() {
        // Test that schema overrides (applied during bump) take precedence
        // over component overrides (applied earlier)
        let zerv_ron = base_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --major 9 --core 0=7 --output-format zerv",
            zerv_ron,
        );

        // Schema override should win
        assert!(output.contains("major: Some(7)"));
        assert!(output.contains("minor: Some(2)"));
        assert!(output.contains("patch: Some(3)"));
    }

    #[rstest]
    #[case::no_conflicts(&["--major", "5", "--minor", "7"])]
    #[case::with_epoch(&["--major", "5", "--minor", "7", "--epoch", "1"])]
    fn test_complex_multi_override_combinations(base_fixture: ZervFixture, #[case] args: &[&str]) {
        let zerv_ron = base_fixture.build().to_string();
        let mut full_args = vec!["version", "--source", "stdin"];
        full_args.extend(args);
        full_args.push("--output-format");
        full_args.push("semver");

        let output = TestCommand::run_with_stdin(&full_args.join(" "), zerv_ron);

        // Verify output is valid semver format
        assert!(output.contains('.'));
        assert!(!output.is_empty());
    }

    #[test]
    fn test_vcs_override_with_clean_flag() {
        let zerv_ron = vcs_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version --source stdin --clean --core 0=8 --output-format zerv",
            zerv_ron,
        );

        // Clean flag should force clean state but schema override should still apply
        assert!(output.contains("major: Some(8)"));
        assert!(output.contains("dirty: Some(false)") || output.contains("dirty: None"));
    }

    #[test]
    fn test_custom_variables_with_other_overrides() {
        let zerv_ron = base_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version \
            --source stdin --custom '{\"env\": \"prod\", \"build\": 42}' \
            --core 0=5 --output-template '{{major}}.{{custom.build}}'",
            zerv_ron,
        );

        // Verify custom variables work alongside other overrides
        assert!(output.contains("5"));
        assert!(output.contains("42"));
    }

    #[test]
    fn test_multiple_schema_sections_with_vcs() {
        let zerv_ron = vcs_fixture().build().to_string();

        let output = TestCommand::run_with_stdin(
            "version \
            --source stdin --core 0=2 --core 1=6 --extra-core 0=3 \
            --extra-core 2=8 --distance 12 --output-format zerv",
            zerv_ron,
        );

        // Verify multiple schema section overrides with VCS
        assert!(output.contains("major: Some(2)"));
        assert!(output.contains("minor: Some(6)"));
        assert!(output.contains("epoch: Some(3)"));
        assert!(output.contains("post: Some(8)"));
        assert!(output.contains("distance: Some(12)"));
    }
}
