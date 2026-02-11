use crate::cli::render::RenderArgs;
use crate::cli::utils::output_formatter::OutputFormatter;
use crate::error::ZervError;
use crate::version::VersionObject;

pub fn run_render(args: RenderArgs) -> Result<String, ZervError> {
    args.validate()?;
    let version_object = VersionObject::parse_with_format(&args.version, &args.input_format)?;
    let zerv = match version_object {
        VersionObject::SemVer(semver) => semver.into(),
        VersionObject::PEP440(pep440) => pep440.into(),
    };
    let output = OutputFormatter::format_output(
        &zerv,
        &args.output.output_format,
        args.output.output_prefix.as_deref(),
        &args.output.output_template,
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::cli::common::args::OutputConfig;
    use crate::cli::utils::template::Template;
    use crate::utils::constants::formats;

    fn create_args(
        version: &str,
        input_format: &str,
        output_format: &str,
        prefix: Option<&str>,
        template: Option<&str>,
    ) -> RenderArgs {
        RenderArgs {
            version: version.to_string(),
            input_format: input_format.to_string(),
            output: OutputConfig {
                output_format: output_format.to_string(),
                output_template: template.map(|s| Template::new(s.to_string())),
                output_prefix: prefix.map(|s| s.to_string()),
            },
        }
    }

    #[rstest]
    #[case("1.2.3", formats::SEMVER, formats::SEMVER, None, None, "1.2.3")]
    #[case("1.2.3", formats::SEMVER, formats::PEP440, None, None, "1.2.3")]
    #[case(
        "1.2.3a1",
        formats::PEP440,
        formats::SEMVER,
        None,
        None,
        "1.2.3-alpha.1"
    )]
    #[case(
        "1.2.3-alpha.1",
        formats::AUTO,
        formats::SEMVER,
        None,
        None,
        "1.2.3-alpha.1"
    )]
    #[case("1.2.3a1", formats::AUTO, formats::PEP440, None, None, "1.2.3a1")]
    #[case("2.0.0", formats::SEMVER, formats::SEMVER, Some("v"), None, "v2.0.0")]
    fn test_run_render_format_conversion(
        #[case] version: &str,
        #[case] input_format: &str,
        #[case] output_format: &str,
        #[case] prefix: Option<&str>,
        #[case] template: Option<&str>,
        #[case] expected: &str,
    ) {
        let args = create_args(version, input_format, output_format, prefix, template);
        let result = run_render(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[rstest]
    // basic template tests
    #[case("1.2.3", formats::AUTO, "{{major}}.{{minor}}", "1.2")]
    #[case("3.4.5", formats::AUTO, "v{{major}}", "v3")]
    #[case(
        "2.1.0",
        formats::AUTO,
        "{{major}}.{{minor}}.{{patch}}-final",
        "2.1.0-final"
    )]
    // all core variables
    #[case(
        "1.2.3-alpha.1",
        formats::AUTO,
        "{{major}}.{{minor}}.{{patch}}",
        "1.2.3"
    )]
    #[case(
        "2.0.0-beta.2",
        formats::AUTO,
        "{{major}}.{{minor}}.{{patch}}-{{pre_release.label}}.{{pre_release.number}}",
        "2.0.0-beta.2"
    )]
    #[case("3.4.5-rc.3", formats::AUTO, "{{semver}}", "3.4.5-rc.3")]
    // semver_obj components
    #[case(
        "1.2.3-alpha.1+build456",
        formats::AUTO,
        "{{semver_obj.base_part}}",
        "1.2.3"
    )]
    #[case("1.2.3-rc.2", formats::AUTO, "{{semver_obj.pre_release_part}}", "rc.2")]
    #[case("1.2.3+build", formats::AUTO, "{{semver_obj.build_part}}", "build")]
    #[case(
        "1.2.3-alpha.1",
        formats::AUTO,
        "{{semver_obj.docker}}",
        "1.2.3-alpha.1"
    )]
    // pep440_obj components (test with auto format detection)
    #[case("1.2.3b2", formats::AUTO, "{{pep440_obj.base_part}}", "1.2.3")]
    #[case("1.2.3a1", formats::AUTO, "{{pep440_obj.pre_release_part}}", "a1")]
    #[case("1.2.3+build", formats::AUTO, "{{pep440_obj.build_part}}", "build")]
    // pre_release context
    #[case("2.0.0-beta.1", formats::AUTO, "{{pre_release.label}}", "beta")]
    #[case("3.0.0-rc.2", formats::AUTO, "{{pre_release.label_code}}", "rc")]
    #[case("1.0.0-alpha.3", formats::AUTO, "{{pre_release.number}}", "3")]
    // complex versions with all components
    #[case(
        "1.2.3b2+build123",
        formats::AUTO,
        "{{major}}.{{minor}}.{{patch}}-{{pre_release.label}}.{{pre_release.number}}+{{semver_obj.build_part}}",
        "1.2.3-beta.2+build123"
    )]
    #[case(
        "1.2.3-rc.1.post.2.dev.3+build456",
        formats::AUTO,
        "{{semver}}",
        "1.2.3-rc.1.post.2.dev.3+build456"
    )]
    #[case(
        "1.2.3a1.post2",
        formats::AUTO,
        "{{major}}.{{minor}}.{{patch}}-{{pre_release.label_code}}.{{pre_release.number}}.{{pep440_obj.pre_release_part}}",
        "1.2.3-a.1.a1.post2"
    )]
    fn test_run_render_with_template(
        #[case] version: &str,
        #[case] input_format: &str,
        #[case] template: &str,
        #[case] expected: &str,
    ) {
        let args = create_args(version, input_format, formats::SEMVER, None, Some(template));
        let result = run_render(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_run_render_template_with_prefix_fails() {
        let args = RenderArgs {
            version: "1.2.3".to_string(),
            input_format: formats::SEMVER.to_string(),
            output: OutputConfig {
                output_format: formats::SEMVER.to_string(),
                output_template: Some(Template::new("v{{major}}".to_string())),
                output_prefix: Some("release-".to_string()),
            },
        };
        let result = run_render(args);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ZervError::ConflictingOptions(_)
        ));
    }

    #[rstest]
    // semver to pep440 (format conversion)
    #[case("1.2.3-alpha.1", formats::SEMVER, formats::PEP440, "1.2.3a1")]
    #[case("1.2.3-beta.2", formats::SEMVER, formats::PEP440, "1.2.3b2")]
    #[case("1.2.3-rc.3", formats::SEMVER, formats::PEP440, "1.2.3rc3")]
    #[case(
        "2.0.0-beta.3+20210101",
        formats::SEMVER,
        formats::PEP440,
        "2.0.0b3+20210101"
    )]
    #[case(
        "3.4.5-rc.1+abc.def.123",
        formats::SEMVER,
        formats::PEP440,
        "3.4.5rc1+abc.def.123"
    )]
    #[case("1.0.0-alpha", formats::SEMVER, formats::PEP440, "1.0.0a0")]
    #[case("5.0.0-beta", formats::SEMVER, formats::PEP440, "5.0.0b0")]
    #[case("2.1.0-rc", formats::SEMVER, formats::PEP440, "2.1.0rc0")]
    // complex semver to pep440 (with post/dev keywords)
    #[case(
        "1.2.3-rc.1.post.2.dev.3+build456",
        formats::SEMVER,
        formats::PEP440,
        "1.2.3rc1.post2.dev3+build456"
    )]
    #[case(
        "1.2.3-alpha.1.internal.post.11.build.dev.2+build456.ci.pipeline.123",
        formats::SEMVER,
        formats::PEP440,
        "1.2.3a1.post11.dev2+internal.build.build456.ci.pipeline.123"
    )]
    #[case(
        "3.0.0-beta.2.feature.branch.name+abc123",
        formats::SEMVER,
        formats::PEP440,
        "3.0.0b2+feature.branch.name.abc123"
    )]
    #[case(
        "1.0.0-alpha.x.y.z+build",
        formats::SEMVER,
        formats::PEP440,
        "1.0.0a0+x.y.z.build"
    )]
    // pep440 to semver (format conversion)
    #[case("1.2.3a1", formats::PEP440, formats::SEMVER, "1.2.3-alpha.1")]
    #[case("1.2.3b2", formats::PEP440, formats::SEMVER, "1.2.3-beta.2")]
    #[case("1.2.3rc3", formats::PEP440, formats::SEMVER, "1.2.3-rc.3")]
    #[case(
        "1.2.3a1.post2",
        formats::PEP440,
        formats::SEMVER,
        "1.2.3-alpha.1.post.2"
    )]
    #[case("1.2.3b2.dev3", formats::PEP440, formats::SEMVER, "1.2.3-beta.2.dev.3")]
    #[case("1.2.3rc1.dev5", formats::PEP440, formats::SEMVER, "1.2.3-rc.1.dev.5")]
    #[case("2!1.2.3", formats::PEP440, formats::SEMVER, "1.2.3-epoch.2")]
    #[case("5!3.0.0a1", formats::PEP440, formats::SEMVER, "3.0.0-epoch.5.alpha.1")]
    #[case(
        "1.2.3a1.post2.dev3",
        formats::PEP440,
        formats::SEMVER,
        "1.2.3-alpha.1.post.2.dev.3"
    )]
    // complex pep440 to semver (edge cases)
    #[case("1.2.3.dev5", formats::PEP440, formats::SEMVER, "1.2.3-dev.5")]
    #[case("1.2.3.post10", formats::PEP440, formats::SEMVER, "1.2.3-post.10")]
    #[case(
        "2.5.0a0.post0.dev0",
        formats::PEP440,
        formats::SEMVER,
        "2.5.0-alpha.0.post.0.dev.0"
    )]
    #[case(
        "3!1.0.0.post2.dev1",
        formats::PEP440,
        formats::SEMVER,
        "1.0.0-epoch.3.post.2.dev.1"
    )]
    #[case(
        "1.0.0a1+local",
        formats::PEP440,
        formats::SEMVER,
        "1.0.0-alpha.1+local"
    )]
    #[case(
        "2.0.0rc5+123.456.abc.def",
        formats::PEP440,
        formats::SEMVER,
        "2.0.0-rc.5+123.456.abc.def"
    )]
    #[case(
        "5.0.0+build.metadata.with.many.parts",
        formats::PEP440,
        formats::SEMVER,
        "5.0.0+build.metadata.with.many.parts"
    )]
    #[case(
        "1.2.3.post1+ubuntu.20.04",
        formats::PEP440,
        formats::SEMVER,
        "1.2.3-post.1+ubuntu.20.4"
    )]
    #[case(
        "3.0.0.dev2+docker.build123",
        formats::PEP440,
        formats::SEMVER,
        "3.0.0-dev.2+docker.build123"
    )]
    #[case(
        "10!5.0.0rc1.post5.dev3+complex.build.metadata",
        formats::PEP440,
        formats::SEMVER,
        "5.0.0-epoch.10.rc.1.post.5.dev.3+complex.build.metadata"
    )]
    // same format (minimal - just verify it works)
    #[case("1.2.3", formats::SEMVER, formats::SEMVER, "1.2.3")]
    #[case("1.2.3+build123", formats::SEMVER, formats::SEMVER, "1.2.3+build123")]
    #[case("1.2.3a1", formats::PEP440, formats::PEP440, "1.2.3a1")]
    #[case("1.2.3rc1.dev5", formats::PEP440, formats::PEP440, "1.2.3rc1.dev5")]
    // with build/local metadata
    #[case(
        "1.2.3-alpha.1+build456",
        formats::SEMVER,
        formats::SEMVER,
        "1.2.3-alpha.1+build456"
    )]
    #[case(
        "1.2.3a1+ubuntu.20.4",
        formats::PEP440,
        formats::PEP440,
        "1.2.3a1+ubuntu.20.4"
    )]
    #[case(
        "2!1.0.0+build.123",
        formats::PEP440,
        formats::PEP440,
        "2!1.0.0+build.123"
    )]
    // complex pre-release (semver preserves as-is)
    #[case(
        "1.2.3-alpha.beta",
        formats::SEMVER,
        formats::SEMVER,
        "1.2.3-alpha.beta"
    )]
    #[case("1.2.3-0.3.7", formats::SEMVER, formats::SEMVER, "1.2.3-0.3.7")]
    #[case(
        "1.2.3-a.b.c.5.d",
        formats::SEMVER,
        formats::SEMVER,
        "1.2.3-alpha.b.c.5.d"
    )]
    fn test_run_render_complex_versions(
        #[case] version: &str,
        #[case] input_format: &str,
        #[case] output_format: &str,
        #[case] expected: &str,
    ) {
        let args = create_args(version, input_format, output_format, None, None);
        let result = run_render(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[rstest]
    #[case("not-a-version", formats::SEMVER)]
    #[case("invalid", formats::PEP440)]
    #[case("", formats::AUTO)]
    fn test_run_render_invalid_input(#[case] version: &str, #[case] input_format: &str) {
        let args = create_args(version, input_format, formats::SEMVER, None, None);
        let result = run_render(args);
        assert!(result.is_err());
    }

    #[rstest]
    #[case("unknown")]
    #[case("invalid")]
    #[case("xyz")]
    fn test_run_render_unknown_input_format(#[case] input_format: &str) {
        let args = create_args("1.0.0", input_format, formats::SEMVER, None, None);
        let result = run_render(args);
        assert!(result.is_err());
    }
}
