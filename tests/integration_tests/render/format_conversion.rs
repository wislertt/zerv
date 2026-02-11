use rstest::rstest;

use crate::util::TestCommand;

mod semver_to_pep440 {
    use super::*;

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3-alpha.1", "1.2.3a1")]
    #[case("1.2.3-beta.2", "1.2.3b2")]
    #[case("1.2.3-rc.3", "1.2.3rc3")]
    #[case("1.0.0-alpha", "1.0.0a0")]
    #[case("2.0.0-beta", "2.0.0b0")]
    fn test_basic_prerelease(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format semver --output-format pep440"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3-alpha.1+build", "1.2.3a1+build")]
    #[case("2.0.0-beta.3+20210101", "2.0.0b3+20210101")]
    #[case("3.4.5-rc.1+abc.def.123", "3.4.5rc1+abc.def.123")]
    fn test_with_build_metadata(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format semver --output-format pep440"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3-rc.1.post.2.dev.3+build456", "1.2.3rc1.post2.dev3+build456")]
    #[case(
        "1.2.3-alpha.1.internal.post.11.build.dev.2+build456",
        "1.2.3a1.post11.dev2+internal.build.build456"
    )]
    fn test_complex_prerelease(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format semver --output-format pep440"
        ));
        assert_eq!(output, expected);
    }
}

mod pep440_to_semver {
    use super::*;

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3a1", "1.2.3-alpha.1")]
    #[case("1.2.3b2", "1.2.3-beta.2")]
    #[case("1.2.3rc3", "1.2.3-rc.3")]
    fn test_basic_prerelease(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format pep440 --output-format semver"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3a1.post2", "1.2.3-alpha.1.post.2")]
    #[case("1.2.3b2.dev3", "1.2.3-beta.2.dev.3")]
    #[case("1.2.3rc1.dev5", "1.2.3-rc.1.dev.5")]
    fn test_with_post_dev(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format pep440 --output-format semver"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("2!1.2.3", "1.2.3-epoch.2")]
    #[case("5!3.0.0a1", "3.0.0-epoch.5.alpha.1")]
    fn test_epoch(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format pep440 --output-format semver"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3.dev5", "1.2.3-dev.5")]
    #[case("1.2.3.post10", "1.2.3-post.10")]
    fn test_dev_without_prerelease(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format pep440 --output-format semver"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3a1+local", "1.2.3-alpha.1+local")]
    #[case("2.0.0rc5+123.456.abc.def", "2.0.0-rc.5+123.456.abc.def")]
    #[case(
        "5.0.0+build.metadata.with.many.parts",
        "5.0.0+build.metadata.with.many.parts"
    )]
    fn test_with_local_version(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format pep440 --output-format semver"
        ));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3a1.post2.dev3", "1.2.3-alpha.1.post.2.dev.3")]
    #[case("3!1.0.0.post2.dev1", "1.0.0-epoch.3.post.2.dev.1")]
    #[case(
        "10!5.0.0rc1.post5.dev3+complex.build.metadata",
        "5.0.0-epoch.10.rc.1.post.5.dev.3+complex.build.metadata"
    )]
    fn test_complex_combinations(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format pep440 --output-format semver"
        ));
        assert_eq!(output, expected);
    }
}

mod same_format {
    use super::*;

    #[rstest]
    #[case("1.2.3", "semver", "1.2.3")]
    #[case("1.2.3+build123", "semver", "1.2.3+build123")]
    #[case("1.2.3a1", "pep440", "1.2.3a1")]
    #[case("1.2.3rc1.dev5", "pep440", "1.2.3rc1.dev5")]
    fn test_same_format_passthrough(
        #[case] input: &str,
        #[case] format: &str,
        #[case] expected: &str,
    ) {
        let output = TestCommand::run(&format!(
            "render {input} --input-format {format} --output-format {format}"
        ));
        assert_eq!(output, expected);
    }
}

mod auto_detect {
    use super::*;

    #[rstest]
    #[case("1.2.3", "1.2.3")]
    #[case("1.2.3-alpha.1", "1.2.3-alpha.1")]
    #[case("1.2.3+build456", "1.2.3+build456")]
    fn test_auto_detect_semver(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input}"));
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("1.2.3a1", "1.2.3a1")]
    #[case("1.2.3b2", "1.2.3b2")]
    #[case("1.2.3rc3", "1.2.3rc3")]
    fn test_auto_detect_pep440(#[case] input: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-format pep440"));
        assert_eq!(output, expected);
    }
}

mod prefix {
    use super::*;

    #[rstest]
    #[case("1.2.3", "v", "v1.2.3")]
    #[case("2.0.0", "release-", "release-2.0.0")]
    fn test_with_prefix(#[case] input: &str, #[case] prefix: &str, #[case] expected: &str) {
        let output = TestCommand::run(&format!("render {input} --output-prefix {prefix}"));
        assert_eq!(output, expected);
    }
}
