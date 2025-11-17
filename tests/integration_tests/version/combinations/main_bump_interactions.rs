//! MainConfig + BumpsConfig interaction tests
//!
//! Tests the interaction between MainConfig options (source, format, schema, template, directory)
//! and BumpsConfig options (primary bumps, secondary bumps, schema bumps, context).

use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;
use zerv::utils::constants::formats::{
    AUTO,
    PEP440,
    SEMVER,
    ZERV,
};

use crate::util::TestCommand;

/// Simple fixture for basic bump interaction tests
#[fixture]
fn simple_fixture() -> ZervFixture {
    ZervFixture::new().with_version(2, 1, 0)
}

/// Fixture with VCS data for VCS-related tests
#[fixture]
fn vcs_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(2, 1, 0)
        .with_distance(5)
        .with_dirty(true)
        .with_branch("feature/test-branch".to_string())
        .with_commit_hash("abc123def456".to_string())
}

/// CalVer fixture for schema-related tests
#[fixture]
fn calver_fixture() -> ZervFixture {
    ZervFixture::new()
        .with_version(1, 2, 3)
        .with_distance(3)
        .with_branch("release".to_string())
        .with_commit_hash("xyz789abc123".to_string())
}

mod source_bump_combinations {
    use super::*;

    #[rstest]
    #[case::stdin_with_bump_major("--bump-major", "3.0.0")]
    #[case::stdin_with_bump_minor("--bump-minor", "2.2.0")]
    #[case::stdin_with_bump_patch("--bump-patch", "2.1.1")]
    fn test_stdin_source_with_primary_bumps(
        simple_fixture: ZervFixture,
        #[case] bump_arg: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = simple_fixture.build().to_string();
        let output =
            TestCommand::run_with_stdin(&format!("version --source stdin {}", bump_arg), zerv_ron);
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case::stdin_with_bump_epoch("--bump-epoch", "0.0.0-epoch.1")]
    #[case::stdin_with_bump_post("--bump-post", "2.1.0-post.1")]
    #[case::stdin_with_bump_dev("--bump-dev", "2.1.0")]
    fn test_stdin_source_with_secondary_bumps(
        simple_fixture: ZervFixture,
        #[case] bump_arg: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = simple_fixture.build().to_string();
        let output =
            TestCommand::run_with_stdin(&format!("version --source stdin {}", bump_arg), zerv_ron);
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_stdin_source_with_multiple_bumps(simple_fixture: ZervFixture) {
        let zerv_ron = simple_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --bump-minor",
            zerv_ron,
        );
        assert_eq!(output, "3.1.0");
    }

    #[rstest]
    fn test_stdin_source_with_custom_bump_values(simple_fixture: ZervFixture) {
        let zerv_ron = simple_fixture.build().to_string();
        let output = TestCommand::run_with_stdin("version --source stdin --bump-major 5", zerv_ron);
        assert_eq!(output, "7.0.0");
    }
}

mod format_bump_combinations {
    use super::*;

    #[rstest]
    #[case::semver_input_bump_major_output_semver(SEMVER, "--bump-major", SEMVER, "3.0.0")]
    #[case::semver_input_bump_minor_output_pep440(SEMVER, "--bump-minor", PEP440, "2.2.0")]
    #[case::auto_input_bump_patch_output_zerv(AUTO, "--bump-patch", ZERV, "major: Some(2)")]
    fn test_format_conversion_with_bumps(
        simple_fixture: ZervFixture,
        #[case] input_format: &str,
        #[case] bump_arg: &str,
        #[case] output_format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = simple_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --input-format {} {} --output-format {}",
                input_format, bump_arg, output_format
            ),
            zerv_ron,
        );

        match output_format {
            ZERV => assert!(output.contains(expected)),
            _ => assert_eq!(output, expected),
        }
    }

    #[rstest]
    fn test_semver_to_pep440_with_bump(simple_fixture: ZervFixture) {
        let zerv_ron = simple_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --input-format semver --bump-major --output-format pep440",
            zerv_ron,
        );
        assert_eq!(output, "3.0.0");
    }

    #[rstest]
    fn test_auto_to_zerv_with_bump(simple_fixture: ZervFixture) {
        let zerv_ron = simple_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --input-format auto --bump-minor --output-format zerv",
            zerv_ron,
        );
        assert!(output.contains("minor: Some(2)"));
    }
}

mod schema_bump_combinations {
    use super::*;

    #[rstest]
    fn test_standard_schema_with_bump_major(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema standard --bump-major --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "3.0.0+feature.test.branch.5.abc123de");
    }

    #[rstest]
    fn test_calver_schema_with_bump_major(calver_fixture: ZervFixture) {
        let zerv_ron = calver_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema calver --bump-major --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "0.0.0+release.3.xyz789ab");
    }

    #[rstest]
    fn test_schema_component_bump_interaction(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        // Test schema with component bump - bump core component 0 (major) by 2
        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema standard --bump-core 0=2 --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "4.0.0+feature.test.branch.5.abc123de");
    }

    #[rstest]
    fn test_schema_with_multiple_bumps(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        // Test schema with multiple bump operations
        let output = TestCommand::run_with_stdin(
            "version --source stdin --schema standard --bump-major --bump-minor --output-format semver",
            zerv_ron,
        );
        assert_eq!(output, "3.1.0+feature.test.branch.5.abc123de");
    }
}

mod template_bump_combinations {
    use super::*;

    #[rstest]
    #[case::basic_template_with_bump_major(
        "v{{major}}.{{minor}}.{{patch}}",
        "--bump-major",
        "v3.0.0"
    )]
    #[case::basic_template_with_bump_minor(
        "{{major}}.{{minor}}.{{patch}}",
        "--bump-minor",
        "2.2.0"
    )]
    #[case::template_with_bump_and_vcs(
        "{{major}}.{{minor}}.{{patch}}+{{distance}}",
        "--bump-patch",
        "2.1.1+5"
    )]
    fn test_template_rendering_with_bumps(
        vcs_fixture: ZervFixture,
        #[case] template: &str,
        #[case] bump_arg: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = vcs_fixture.build().to_string();
        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --output-template '{}' {}",
                template, bump_arg
            ),
            zerv_ron,
        );
        assert_eq!(output, expected);
    }

    #[rstest]
    fn test_template_with_bump_context(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        // Test template with bump context preservation (default behavior)
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --output-template '{{major}}.{{minor}}.{{patch}}+{{distance}}'",
            zerv_ron,
        );
        assert_eq!(output, "3.0.0+5");
    }

    #[rstest]
    fn test_template_with_no_bump_context(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        // Test template with no bump context (VCS data cleared)
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --no-bump-context --output-template '{{major}}.{{minor}}.{{patch}}+{{distance}}'",
            zerv_ron,
        );
        assert_eq!(output, "3.0.0+0");
    }

    #[rstest]
    fn test_template_with_sanitize_helper_and_bumps(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();

        // Test sanitize helper with bumped version
        let output = TestCommand::run_with_stdin(
            "version --source stdin --bump-major --bumped-branch 'feature/test' --output-template '{{ sanitize(value=bumped_branch, preset=\"dotted\") }}-v{{major}}'",
            zerv_ron,
        );
        assert_eq!(output, "feature.test-v3");
    }

    #[rstest]
    fn test_simple_template_with_multiple_bumps(vcs_fixture: ZervFixture) {
        let zerv_ron = vcs_fixture.build().to_string();
        let template = "build-{{major}}.{{minor}}.{{patch}}+{{distance}}";

        let output = TestCommand::run_with_stdin(
            &format!(
                "version --source stdin --bump-major --output-template '{}'",
                template
            ),
            zerv_ron,
        );
        assert_eq!(output, "build-3.0.0+5");
    }
}
