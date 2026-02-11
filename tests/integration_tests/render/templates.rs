use rstest::rstest;

use crate::util::TestCommand;

mod basic_templates {
    use super::*;

    #[rstest]
    #[case("1.2.3", "{{major}}.{{minor}}", "1.2")]
    #[case("3.4.5", "v{{major}}", "v3")]
    #[case("2.1.0", "{{major}}.{{minor}}.{{patch}}-final", "2.1.0-final")]
    fn test_basic_variables(#[case] input: &str, #[case] template: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }
}

mod prerelease_templates {
    use super::*;

    #[rstest]
    #[case(
        "2.0.0-beta.2",
        "{{major}}.{{minor}}.{{patch}}-{{pre_release.label}}.{{pre_release.number}}",
        "2.0.0-beta.2"
    )]
    #[case("1.0.0-alpha.3", "{{pre_release.label}}", "alpha")]
    #[case("3.0.0-rc.2", "{{pre_release.label_code}}", "rc")]
    fn test_prerelease_variables(
        #[case] input: &str,
        #[case] template: &str,
        #[case] expected: &str,
    ) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }
}

mod object_templates {
    use super::*;

    #[rstest]
    #[case("1.2.3-alpha.1+build456", "{{semver_obj.base_part}}", "1.2.3")]
    #[case("1.2.3-rc.2", "{{semver_obj.pre_release_part}}", "rc.2")]
    #[case("1.2.3+build", "{{semver_obj.build_part}}", "build")]
    fn test_semver_obj(#[case] input: &str, #[case] template: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3b2", "{{pep440_obj.base_part}}", "1.2.3")]
    #[case("1.2.3a1", "{{pep440_obj.pre_release_part}}", "a1")]
    #[case("1.2.3+build", "{{pep440_obj.build_part}}", "build")]
    fn test_pep440_obj(#[case] input: &str, #[case] template: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }
}

mod full_version_templates {
    use super::*;

    #[rstest]
    #[case("1.2.3-alpha.1", "{{semver}}", "1.2.3-alpha.1")]
    #[case("1.2.3b2", "{{pep440}}", "1.2.3b2")]
    fn test_full_version(#[case] input: &str, #[case] template: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }
}

mod pep440_extended_templates {
    use super::*;

    #[rstest]
    #[case(
        "1.2.3.post5",
        "{{major}}.{{minor}}.{{patch}}-post.{{post}}",
        "1.2.3-post.5"
    )]
    #[case(
        "1.2.3.dev3",
        "{{major}}.{{minor}}.{{patch}}-dev.{{dev}}",
        "1.2.3-dev.3"
    )]
    #[case(
        "1.2.3",
        "{% if epoch %}{{epoch}}!{% endif %}{{major}}.{{minor}}.{{patch}}",
        "1.2.3"
    )]
    fn test_post_dev_epoch(#[case] input: &str, #[case] template: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }
}

mod template_with_format_conversion {
    use super::*;

    #[rstest]
    #[case("1.2.3-alpha.1", "{{pep440}}", "1.2.3a1")]
    fn test_template_uses_pep440_variable(
        #[case] input: &str,
        #[case] template: &str,
        #[case] expected: &str,
    ) {
        let output = TestCommand::run(&format!("render {input} --output-template '{template}'"));
        assert_eq!(output, expected);
    }
}

mod validation {
    use super::*;

    #[test]
    fn test_template_with_prefix_fails() {
        let output = TestCommand::run_expect_fail(
            "render 1.2.3 --output-template 'v{{major}}' --output-prefix 'release-'",
        );
        assert!(output.contains("conflicting") || output.contains("template"));
    }

    #[test]
    fn test_template_with_non_semver_fails() {
        let output = TestCommand::run_expect_fail(
            "render 1.2.3 --output-template 'v{{major}}' --output-format pep440",
        );
        assert!(output.contains("conflicting") || output.contains("template"));
    }
}
