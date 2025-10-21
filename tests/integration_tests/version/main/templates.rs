use rstest::rstest;
use zerv::test_utils::ZervFixture;
use zerv::version::PreReleaseLabel;

use crate::util::TestCommand;

fn run_template(template: &str, fixture: ZervFixture) -> String {
    let zerv_ron = fixture.build().to_string();
    TestCommand::new()
        .args_from_str(format!(
            "version --source stdin --output-template '{template}'"
        ))
        .stdin(zerv_ron)
        .assert_success()
        .stdout()
        .trim()
        .to_string()
}

mod template_basic_variables {
    use super::*;

    #[rstest]
    #[case::major_only("{{major}}", (1, 2, 3), "1")]
    #[case::minor_only("{{minor}}", (1, 2, 3), "2")]
    #[case::patch_only("{{patch}}", (1, 2, 3), "3")]
    #[case::major_minor("{{major}}.{{minor}}", (1, 2, 3), "1.2")]
    #[case::full_version("{{major}}.{{minor}}.{{patch}}", (1, 2, 3), "1.2.3")]
    #[case::custom_separator("{{major}}_{{minor}}_{{patch}}", (1, 2, 3), "1_2_3")]
    #[case::with_prefix("v{{major}}.{{minor}}.{{patch}}", (1, 2, 3), "v1.2.3")]
    #[case::with_suffix("{{major}}.{{minor}}.{{patch}}-release", (2, 0, 0), "2.0.0-release")]
    fn test_template_basic(
        #[case] template: &str,
        #[case] version: (u64, u64, u64),
        #[case] expected: &str,
    ) {
        let fixture = ZervFixture::new().with_version(version.0, version.1, version.2);
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_extended_fields {
    use super::*;

    #[rstest]
    #[case::epoch("{{epoch}}!{{major}}.{{minor}}.{{patch}}", |f: ZervFixture| f.with_epoch(2), "2!1.2.3")]
    #[case::post("{{major}}.{{minor}}.{{patch}}.post{{post}}", |f: ZervFixture| f.with_post(5), "1.2.3.post5")]
    #[case::dev("{{major}}.{{minor}}.{{patch}}.dev{{dev}}", |f: ZervFixture| f.with_dev(1234567890), "1.2.3.dev1234567890")]
    fn test_template_extended(
        #[case] template: &str,
        #[case] setup: fn(ZervFixture) -> ZervFixture,
        #[case] expected: &str,
    ) {
        let fixture = setup(ZervFixture::new().with_version(1, 2, 3));
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_prerelease {
    use super::*;

    #[rstest]
    #[case::alpha(PreReleaseLabel::Alpha, Some(1), "1.0.0-alpha.1")]
    #[case::beta(PreReleaseLabel::Beta, Some(2), "1.0.0-beta.2")]
    #[case::rc(PreReleaseLabel::Rc, Some(3), "1.0.0-rc.3")]
    fn test_template_prerelease(
        #[case] label: PreReleaseLabel,
        #[case] number: Option<u64>,
        #[case] expected: &str,
    ) {
        let template = "{{major}}.{{minor}}.{{patch}}-{{pre_release.label}}.{{pre_release.number}}";
        let fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_pre_release(label, number);
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_vcs_fields {
    use super::*;

    #[rstest]
    #[case::distance(
        "{{major}}.{{minor}}.{{patch}}+{{distance}}",
        |f: ZervFixture| f.with_vcs_data(Some(5), Some(false), None, None, None, None, None),
        "1.0.0+5"
    )]
    #[case::branch(
        "{{major}}.{{minor}}.{{patch}}+{{bumped_branch}}",
        |f: ZervFixture| f.with_vcs_data(None, None, Some("feature.branch".to_string()), None, None, None, None),
        "1.0.0+feature.branch"
    )]
    #[case::commit_hash(
        "{{major}}.{{minor}}.{{patch}}+{{bumped_commit_hash}}",
        |f: ZervFixture| f.with_vcs_data(None, None, None, Some("abc123def456".to_string()), None, None, None),
        "1.0.0+abc123def456"
    )]
    #[case::commit_hash_short(
        "{{major}}.{{minor}}.{{patch}}+{{bumped_commit_hash_short}}",
        |f: ZervFixture| f.with_vcs_data(None, None, None, Some("abc123def456".to_string()), None, None, None),
        "1.0.0+abc123d"
    )]
    fn test_template_vcs(
        #[case] template: &str,
        #[case] setup: fn(ZervFixture) -> ZervFixture,
        #[case] expected: &str,
    ) {
        let fixture = setup(ZervFixture::new().with_version(1, 0, 0));
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_formatted_versions {
    use super::*;

    #[rstest]
    #[case::semver("{{semver}}", "1.2.3-alpha.1")]
    #[case::pep440("{{pep440}}", "1.2.3a1")]
    #[case::both(
        "SemVer: {{semver}}, PEP440: {{pep440}}",
        "SemVer: 1.2.3-alpha.1, PEP440: 1.2.3a1"
    )]
    fn test_template_formatted(#[case] template: &str, #[case] expected: &str) {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_pre_release(PreReleaseLabel::Alpha, Some(1));
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_helpers_sanitize {
    use super::*;

    #[rstest]
    #[case::dotted("Feature/API-v2", "dotted", "Feature.API.v2")]
    #[case::semver("Build-ID-0051", "semver", "Build.ID.51")]
    #[case::lower_dotted("Feature/API-v2", "lower_dotted", "feature.api.v2")]
    #[case::pep440("Build-ID-0051", "pep440", "build.id.51")]
    fn test_sanitize_presets(#[case] input: &str, #[case] preset: &str, #[case] expected: &str) {
        let template = format!("{{{{sanitize bumped_branch preset=\"{preset}\"}}}}");
        let fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some(input.to_string()),
            None,
            None,
            None,
            None,
        );
        assert_eq!(run_template(&template, fixture), expected);
    }

    #[rstest]
    #[case::custom_separator(
        "feature-branch",
        "{{sanitize bumped_branch separator=\"_\"}}",
        "feature_branch"
    )]
    #[case::max_length(
        "VeryLongBranchName",
        "{{sanitize bumped_branch max_length=10 lowercase=false}}",
        "VeryLongBr"
    )]
    fn test_sanitize_custom(#[case] input: &str, #[case] template: &str, #[case] expected: &str) {
        let fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some(input.to_string()),
            None,
            None,
            None,
            None,
        );
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_helpers_hash {
    use super::*;

    #[rstest]
    #[case::default("{{hash bumped_branch}}", "c7dedb4")]
    #[case::custom_length("{{hash bumped_branch 10}}", "c7dedb4632")]
    #[case::hash_int("{{hash_int bumped_branch}}", "7126668")]
    fn test_hash(#[case] template: &str, #[case] expected: &str) {
        let fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            Some("test".to_string()),
            None,
            None,
            None,
            None,
        );
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_helpers_prefix {
    use super::*;

    #[test]
    fn test_prefix() {
        let template = "{{prefix bumped_commit_hash 7}}";
        let fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            None,
            Some("abcdef123456789".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(run_template(template, fixture), "abcdef1");
    }
}

mod template_helpers_timestamp {
    use super::*;

    #[rstest]
    #[case::default("{{format_timestamp last_timestamp}}", "2023-12-21")]
    #[case::compact_date(
        "{{format_timestamp last_timestamp format=\"compact_date\"}}",
        "20231221"
    )]
    #[case::compact_datetime(
        "{{format_timestamp last_timestamp format=\"compact_datetime\"}}",
        "20231221015056"
    )]
    fn test_timestamp(#[case] template: &str, #[case] expected: &str) {
        let fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            None,
            None,
            None,
            Some(1703123456),
            None,
        );
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_helpers_math {
    use super::*;

    #[rstest]
    #[case::add("{{add major minor}}", (1, 2, 3), "3")]
    #[case::subtract("{{subtract major minor}}", (5, 2, 0), "3")]
    #[case::multiply("{{multiply major minor}}", (3, 4, 0), "12")]
    fn test_math(#[case] template: &str, #[case] version: (u64, u64, u64), #[case] expected: &str) {
        let fixture = ZervFixture::new().with_version(version.0, version.1, version.2);
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_complex_scenarios {
    use super::*;

    #[test]
    fn test_tier_2_pattern() {
        let template = "{{major}}.{{minor}}.{{patch}}.post{{distance}}+{{bumped_branch}}.{{bumped_commit_hash_short}}";
        let fixture = ZervFixture::new().with_version(1, 2, 3).with_vcs_data(
            Some(5),
            Some(false),
            Some("main".to_string()),
            Some("abc123".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(run_template(template, fixture), "1.2.3.post5+main.abc123");
    }

    #[test]
    fn test_tier_3_pattern() {
        let template = "{{major}}.{{minor}}.{{patch}}.dev{{dev}}+{{sanitize bumped_branch}}.{{bumped_commit_hash_short}}";
        let fixture = ZervFixture::new()
            .with_version(1, 0, 0)
            .with_dev(1234567890)
            .with_vcs_data(
                Some(0),
                Some(true),
                Some("feature.branch".to_string()),
                Some("def456".to_string()),
                None,
                None,
                None,
            );
        assert_eq!(
            run_template(template, fixture),
            "1.0.0.dev1234567890+feature.branch.def456"
        );
    }

    #[test]
    fn test_multiple_helpers() {
        let template = "{{major}}.{{minor}}.{{patch}}-{{sanitize bumped_branch preset=\"dotted\"}}.{{format_timestamp last_timestamp format=\"compact_date\"}}";
        let fixture = ZervFixture::new().with_version(2, 1, 0).with_vcs_data(
            None,
            None,
            Some("Feature/API-v2".to_string()),
            None,
            None,
            Some(1703123456),
            None,
        );
        assert_eq!(
            run_template(template, fixture),
            "2.1.0-Feature.API.v2.20231221"
        );
    }

    #[test]
    fn test_calver_pattern() {
        let template = "{{format_timestamp last_timestamp format=\"%Y\"}}.{{format_timestamp last_timestamp format=\"%m\"}}.{{major}}";
        let fixture = ZervFixture::new().with_version(1, 0, 0).with_vcs_data(
            None,
            None,
            None,
            None,
            None,
            Some(1703123456),
            None,
        );
        assert_eq!(run_template(template, fixture), "2023.12.1");
    }
}

mod template_edge_cases {
    use super::*;

    #[rstest]
    #[case::missing_vars(
        "{{major}}.{{minor}}.{{patch}}{{#if epoch}}-epoch{{/if}}",
        |f: ZervFixture| f,
        "1.0.0"
    )]
    #[case::conditional(
        "{{#if epoch}}{{epoch}}!{{/if}}{{major}}.{{minor}}.{{patch}}",
        |f: ZervFixture| f.with_epoch(2),
        "2!1.0.0"
    )]
    #[case::static_text(
        "static-version-1.0.0",
        |f: ZervFixture| f,
        "static-version-1.0.0"
    )]
    fn test_edge_cases(
        #[case] template: &str,
        #[case] setup: fn(ZervFixture) -> ZervFixture,
        #[case] expected: &str,
    ) {
        let fixture = setup(ZervFixture::new().with_version(1, 0, 0));
        assert_eq!(run_template(template, fixture), expected);
    }
}

mod template_with_schema {
    use super::*;

    #[test]
    fn test_template_overrides_schema() {
        let fixture = ZervFixture::new()
            .with_version(1, 2, 3)
            .with_standard_tier_1();
        let zerv_ron = fixture.build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --schema zerv-standard --output-template '{{major}}.{{minor}}'")
            .stdin(zerv_ron)
            .assert_success();

        assert_eq!(
            output.stdout().trim(),
            "1.2",
            "Template should override schema output"
        );
    }
}

mod template_validation_errors {
    use super::*;

    #[test]
    fn test_template_conflicts_with_output_format() {
        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv_ron = fixture.build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --output-format pep440 --output-template '{{major}}.{{minor}}.{{patch}}'")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("Cannot use --output-template with --output-format"),
            "Should error when using --output-template with --output-format. Got stderr: {stderr}"
        );
        assert!(
            stderr.contains("Use --output-format alone for pure format output"),
            "Error message should explain the distinction between format and template. Got stderr: {stderr}"
        );
    }

    #[test]
    fn test_template_conflicts_with_output_prefix() {
        let fixture = ZervFixture::new().with_version(1, 2, 3);
        let zerv_ron = fixture.build().to_string();

        let output = TestCommand::new()
            .args_from_str("version --source stdin --output-template '{{major}}.{{minor}}.{{patch}}' --output-prefix v")
            .stdin(zerv_ron)
            .assert_failure();

        let stderr = output.stderr();
        assert!(
            stderr.contains("Cannot use --output-template with --output-prefix"),
            "Should error when using --output-template with --output-prefix. Got stderr: {stderr}"
        );
        assert!(
            stderr.contains("Add the prefix directly in your template"),
            "Error message should suggest adding prefix in template. Got stderr: {stderr}"
        );
    }
}
