use rstest::{
    fixture,
    rstest,
};
use zerv::test_utils::ZervFixture;

use crate::util::TestCommand;

#[fixture]
fn base_fixture() -> ZervFixture {
    ZervFixture::new().with_version(1, 2, 3)
}

mod basic_json_parsing {
    use super::*;

    #[rstest]
    fn test_custom_simple_string() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"build_id":"abc123"}' "#,
                r#"--output-template "{{custom.build_id}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "abc123");
    }

    #[rstest]
    fn test_custom_number() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"build_num":42}' "#,
                r#"--output-template "{{custom.build_num}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "42");
    }

    #[rstest]
    fn test_custom_boolean() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"enabled":true}' "#,
                r#"--output-template "{{custom.enabled}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "true");
    }

    #[rstest]
    fn test_custom_multiple_fields() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"env":"prod","region":"us-east-1"}' "#,
                r#"--output-template "{{custom.env}}-{{custom.region}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "prod-us-east-1");
    }
}

mod nested_json {
    use super::*;

    #[rstest]
    fn test_custom_nested_object() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"metadata":{"author":"ci","env":"prod"}}' "#,
                r#"--output-template "{{custom.metadata.author}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "ci");
    }

    #[rstest]
    fn test_custom_deeply_nested() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"config":{"database":{"host":"localhost"}}}' "#,
                r#"--output-template "{{custom.config.database.host}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "localhost");
    }

    #[rstest]
    fn test_custom_nested_multiple_levels() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"metadata":{"author":"ci","timestamp":1703123456}}' "#,
                r#"--output-template "{{custom.metadata.author}}-{{custom.metadata.timestamp}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "ci-1703123456");
    }
}

mod combined_with_version {
    use super::*;

    #[rstest]
    fn test_custom_with_version_fields(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"build":"ci123"}' "#,
                r#"--output-template "{{major}}.{{minor}}.{{patch}}+{{custom.build}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "1.2.3+ci123");
    }

    #[rstest]
    fn test_custom_with_semver_output(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --custom '{"env":"prod"}' --output-format semver"#,
            zerv_ron,
        );

        assert_eq!(output, "1.2.3");
    }

    #[rstest]
    fn test_custom_with_pep440_output(base_fixture: ZervFixture) {
        let zerv_ron = base_fixture.build().to_string();

        let output = TestCommand::run_with_stdin(
            r#"version --source stdin --custom '{"env":"prod"}' --output-format pep440"#,
            zerv_ron,
        );

        assert_eq!(output, "1.2.3");
    }
}

mod combined_with_vcs {
    use super::*;

    #[rstest]
    fn test_custom_with_branch() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_vcs_data(None, None, Some("main".to_string()), None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"build":"123"}' "#,
                r#"--output-template "{{custom.build}}-{{bumped_branch}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "123-main");
    }

    #[rstest]
    fn test_custom_with_distance_and_dirty() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_vcs_data(Some(5), Some(true), None, None, None, None, None)
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"env":"dev"}' "#,
                "--output-template ",
                r#""{{major}}.{{minor}}.{{patch}}+{{custom.env}}.{{distance}}.dirty""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "1.0.0+dev.5.dirty");
    }
}

mod error_handling {
    use super::*;

    #[rstest]
    fn test_invalid_json() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        TestCommand::new()
            .args([
                "version",
                "--source",
                "stdin",
                "--custom",
                r#"{"invalid": json}"#,
            ])
            .stdin(zerv_ron)
            .assert_failure()
            .assert_stderr_contains("Invalid custom JSON");
    }

    #[rstest]
    fn test_missing_custom_key() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            // r#"version --source stdin --custom '{"build":"123"}' \
            //    --output-template "{{custom.nonexistent}}""#,
            concat!(
                "version --source stdin ",
                r#"--custom '{"build":"123"}' "#,
                "--output-template '{{custom.nonexistent}}'"
            ),
            zerv_ron,
        );

        assert_eq!(output, "");
    }

    #[rstest]
    fn test_unsupported_type_array() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"tags":["v1","v2"]}' "#,
                r#"--output-template "{{custom.tags}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "[v1, v2]");
    }

    #[rstest]
    fn test_unsupported_type_null() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"value":null}' "#,
                r#"--output-template "{{custom.value}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "");
    }
}

mod template_helpers {
    use super::*;

    #[rstest]
    fn test_custom_with_sanitize() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"branch":"feature/test-branch"}' "#,
                r#"--output-template "{{sanitize custom.branch separator=\"_\"}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "feature_test_branch");
    }

    #[rstest]
    fn test_custom_with_hash() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"env":"production"}' "#,
                r#"--output-template "{{hash custom.env}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output.len(), 7);
    }

    #[rstest]
    fn test_custom_with_prefix() {
        let zerv_ron = ZervFixture::new().with_version(1, 0, 0).build().to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"build":"abc123def"}' "#,
                r#"--output-template "{{prefix custom.build 3}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "abc");
    }
}

mod real_world_scenarios {
    use super::*;

    #[rstest]
    fn test_ci_build_metadata() {
        let zerv_ron = ZervFixture::new()
            .with_version(2, 1, 0)
            .with_vcs_data(
                Some(0),
                Some(false),
                Some("main".to_string()),
                None,
                None,
                None,
                None,
            )
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"ci_build":"456","ci_runner":"github"}' "#,
                "--output-template ",
                r#""{{major}}.{{minor}}.{{patch}}+{{custom.ci_runner}}.{{custom.ci_build}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "2.1.0+github.456");
    }

    #[rstest]
    fn test_deployment_metadata() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 5, 2)
            .with_vcs_data(
                None,
                None,
                Some("production".to_string()),
                None,
                None,
                None,
                None,
            )
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"env":"prod","region":"us-east-1","az":"1a"}' "#,
                "--output-template ",
                r#""{{major}}.{{minor}}.{{patch}}-{{custom.env}}-{{custom.region}}-{{custom.az}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "1.5.2-prod-us-east-1-1a");
    }

    #[rstest]
    fn test_docker_tag_format() {
        let zerv_ron = ZervFixture::new()
            .with_version(3, 0, 0)
            .with_vcs_data(
                Some(0),
                Some(false),
                Some("main".to_string()),
                Some("abc123def456".to_string()),
                None,
                None,
                None,
            )
            .build()
            .to_string();

        let output = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin ",
                r#"--custom '{"registry":"ghcr.io","org":"myorg","repo":"myapp"}' "#,
                "--output-template ",
                r#""{{custom.registry}}/{{custom.org}}/{{custom.repo}}:"#,
                r#"{{major}}.{{minor}}.{{patch}}-{{bumped_commit_hash_short}}""#
            ),
            zerv_ron,
        );

        assert_eq!(output, "ghcr.io/myorg/myapp:3.0.0-abc123d");
    }
}
