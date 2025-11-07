use rstest::rstest;
use zerv::schema::ZervSchemaPreset;
use zerv::test_utils::{
    ZervFixture,
    ZervSchemaFixture,
};
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

mod schema_format {
    use super::*;

    #[rstest]
    #[case::standard_semver("standard", "semver", "1.2.3")]
    #[case::standard_pep440("standard", "pep440", "1.2.3")]
    #[case::calver_semver("calver", "semver", "21.0.0")]
    #[case::calver_pep440("calver", "pep440", "21")]
    fn test_preset_schema_with_output_format(
        #[case] schema: &str,
        #[case] format: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = if schema == "calver" {
            ZervFixture::new()
                .with_version(2025, 10, 21)
                .with_schema_preset(ZervSchemaPreset::CalverBasePrerelease)
                .build()
                .to_string()
        } else {
            ZervFixture::new()
                .with_version(1, 2, 3)
                .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
                .build()
                .to_string()
        };

        let result = TestCommand::run_with_stdin(
            &format!("version --source stdin --schema {schema} --output-format {format}"),
            zerv_ron,
        );

        assert_eq!(result.trim(), expected);
    }

    #[rstest]
    #[case::epoch_pep440("3!1.2", "pep440")]
    #[case::epoch_semver("1.2.0-epoch.3", "semver")]
    fn test_custom_schema_with_epoch_and_format(#[case] expected: &str, #[case] format: &str) {
        let schema_ron = ZervSchemaFixture::empty()
            .with_major_minor()
            .with_epoch_in_extra_core()
            .build()
            .to_string();

        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 0)
            .with_epoch(3)
            .build()
            .to_string();

        let result = TestCommand::run_with_stdin(
            &format!("version --source stdin --schema-ron '{schema_ron}' --output-format {format}"),
            zerv_ron,
        );

        assert_eq!(result.trim(), expected);
    }
}

mod schema_template {
    use super::*;

    #[rstest]
    #[case::standard("standard", "v{{major}}.{{minor}}.{{patch}}", "v1.2.3")]
    #[case::calver("calver", "{{major}}-{{minor}}-{{patch}}", "2025-10-21")]
    fn test_preset_schema_with_template(
        #[case] schema: &str,
        #[case] template: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = if schema == "calver" {
            ZervFixture::new()
                .with_version(2025, 10, 21)
                .with_schema_preset(ZervSchemaPreset::CalverBasePrerelease)
                .build()
                .to_string()
        } else {
            ZervFixture::new()
                .with_version(1, 2, 3)
                .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
                .build()
                .to_string()
        };

        let result = TestCommand::run_with_stdin(
            &format!(r#"version --source stdin --schema {schema} --output-template "{template}""#),
            zerv_ron,
        );

        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_custom_schema_with_template_and_helpers() {
        let schema_ron = ZervSchemaFixture::empty()
            .with_major_minor_patch()
            .with_epoch_in_extra_core()
            .build()
            .to_string();

        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_epoch(2)
            .with_vcs_data(
                None,
                None,
                Some("feature/test".to_string()),
                None,
                None,
                None,
                None,
            )
            .build()
            .to_string();

        let cmd = format!(
            concat!(
                "version --source stdin --schema-ron '{}' --output-template ",
                r#""{{{{epoch}}}}:{{{{semver}}}}-{{{{ sanitize(value=bumped_branch) }}}}""#
            ),
            schema_ron
        );
        let result = TestCommand::run_with_stdin(&cmd, zerv_ron);

        assert_eq!(result.trim(), "2:1.2.3-epoch.2-feature.test");
    }
}

mod format_prefix {
    use super::*;

    #[rstest]
    #[case::semver("semver", "v", "v1.2.3")]
    #[case::pep440("pep440", "v", "v1.2.3")]
    #[case::semver_release("semver", "release-", "release-1.2.3")]
    fn test_output_format_with_prefix(
        #[case] format: &str,
        #[case] prefix: &str,
        #[case] expected: &str,
    ) {
        let zerv_ron = ZervFixture::new().with_version(1, 2, 3).build().to_string();

        let result = TestCommand::run_with_stdin(
            &format!(
                r#"version --source stdin --output-format {format} --output-prefix "{prefix}""#
            ),
            zerv_ron,
        );

        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn test_epoch_with_prefix_pep440() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_epoch(5)
            .build()
            .to_string();

        let result = TestCommand::run_with_stdin(
            "version --source stdin --output-format pep440 --output-prefix v",
            zerv_ron,
        );

        assert_eq!(result.trim(), "v5!1.2.3");
    }
}

mod template_helpers {
    use super::*;

    #[test]
    fn test_multiple_format_helpers_in_template() {
        let zerv_ron = ZervFixture::new()
            .with_version(3, 2, 1)
            .with_pre_release(PreReleaseLabel::Rc, Some(2))
            .build()
            .to_string();

        let result = TestCommand::run_with_stdin(
            concat!(
                "version --source stdin --output-template ",
                r#""SemVer={{semver}},PEP440={{pep440}},Raw={{major}}.{{minor}}.{{patch}}""#
            ),
            zerv_ron,
        );

        assert_eq!(result.trim(), "SemVer=3.2.1-rc.2,PEP440=3.2.1rc2,Raw=3.2.1");
    }

    #[test]
    fn test_template_with_sanitize_helper() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_vcs_data(
                None,
                None,
                Some("feature/test-branch".to_string()),
                None,
                None,
                None,
                None,
            )
            .build()
            .to_string();

        let result = TestCommand::run_with_stdin(
            r#"version --source stdin --output-template "{{semver}}-{{ sanitize(value=bumped_branch) }}""#,
            zerv_ron,
        );

        assert_eq!(result.trim(), "1.0.0-feature.test.branch");
    }
}

mod multi_option {
    use super::*;

    #[test]
    fn test_custom_schema_template_complex_workflow() {
        let schema_ron = ZervSchemaFixture::empty()
            .with_major_minor_patch()
            .build()
            .to_string();

        let zerv_ron = ZervFixture::new().with_version(2, 5, 1).build().to_string();

        let cmd = format!(
            concat!(
                "version --source stdin --schema-ron '{}' --output-template ",
                r#""release-{{{{major}}}}.{{{{minor}}}}.{{{{patch}}}}""#
            ),
            schema_ron
        );
        let result = TestCommand::run_with_stdin(&cmd, zerv_ron);

        assert_eq!(result.trim(), "release-2.5.1");
    }

    #[test]
    fn test_schema_format_prefix_together() {
        let zerv_ron = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_schema_preset(ZervSchemaPreset::StandardBasePrerelease)
            .build()
            .to_string();

        let result = TestCommand::run_with_stdin(
            "version --source stdin --schema standard --output-format semver --output-prefix v",
            zerv_ron,
        );

        assert_eq!(result.trim(), "v1.0.0");
    }

    #[rstest]
    #[case::post_only("2.0.0.post3")]
    fn test_extended_version_with_template(#[case] expected: &str) {
        let zerv_ron = ZervFixture::new()
            .with_version(2, 0, 0)
            .with_post(3)
            .build()
            .to_string();

        let result = TestCommand::run_with_stdin(
            r#"version --source stdin --output-template "{{pep440}}""#,
            zerv_ron,
        );

        assert_eq!(result.trim(), expected);
    }
}
